//! What the client draws, in world space: the free runner's game
//! (`rondata::free`), the heightfield it reports, and each drawn figure's
//! decoded mesh at its draw-list anchor and facing (`docs/RENDERER.md`
//! §3.6). Nothing here touches the GPU; `gpu` takes the triangles.

use render::bh3;
use render::draw::{Drawn, MODELS_AT_THE_POINT, Placed, PlacedBuilding, Pose, SpriteAt, sprite_at};
use render::figure::{apply, guy_placement, placement, posed, rest_pose};
use render::terrain::{Heightfield, TerrainMesh};
use rondata::artdata::{
    BuildLook, ConstructionArt, ScaffoldArt, build_art, construction_art, resolve_art,
    scaffold_art, unit_art,
};
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

/// **Which game the client stands up** at frame zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Game {
    /// The pair's: the long capture's start, its sibling's blocks and its
    /// pasture's trace (`FreeRunner::from_captures`).
    Pair,
    /// The minted script's lobby, built from its choices alone
    /// (`FreeRunner::from_lobby`, item 4145): our own start.
    Lobby,
}

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
    /// The `<UNIT>`'s `scale=` times [`GUY_SCALE`]; 1 for a building;
    /// [`SCAFFOLD_SCALE`] for a scaffold piece.
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

/// **The scale `GraphicPieces::draw_scaffold@00907420` draws a scaffold
/// piece at** (item 4163): its `GraphicChad` takes `sx` 3, `sy` −3, `sz` 3
/// — a guy's mirror, three times over — and the slot's `angle` as it
/// stands, with no turn added (`GuyOut::draw` adds 180 to a figure's).
pub const SCAFFOLD_SCALE: f32 = 3.0;

/// The client's key for the `<CONSTRUCTION>` entry at `slot`
/// ([`render::draw::Foundation::slot`]) where a piece number keys art
/// (item 4163). No reader knows `first_construct_piece`, so the key is
/// the client's own: below every game piece, which is never negative but
/// `-1`.
pub fn foundation_key(slot: u8) -> i32 {
    FOUNDATION_KEY - i32::from(slot)
}

/// The client's key for the `<SCAFFOLD>` entry at `element`
/// ([`render::draw::ScaffoldPiece::element`]), as [`foundation_key`].
pub fn scaffold_key(element: u8) -> i32 {
    SCAFFOLD_KEY - i32::from(element)
}

const FOUNDATION_KEY: i32 = -0x100;
const SCAFFOLD_KEY: i32 = -0x200;

/// The slot or element a client key names, where it is one.
fn keyed(piece: i32, base: i32) -> Option<usize> {
    let i = base - piece;
    (0..0x100).contains(&i).then_some(i as usize)
}

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
    /// The shadows' art (item 4182).
    shadow_art: ShadowArt,
    /// Draw the shadows (item 4182). On by default; the row's count as
    /// drawn before turns it off.
    pub shadows: bool,
    /// Draw the guys' shadows too (item 4182): off by default. As read
    /// (`draw_unit_shadow`), they darken the row's figure tiles past the
    /// photograph's, 28 → 5 of 37 on brightness (parked 4182c); the row's
    /// selection keeps them either way.
    pub unit_shadows: bool,
    /// The shadows' row's made-to-fail (item 4182): every decal's x offset
    /// with its sign turned, the shadow mirrored about its point in x.
    pub shadow_mirror: bool,
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
    /// resolve or decode; a foundation's under its [`foundation_key`].
    sprite_images: BTreeMap<i32, Option<std::sync::Arc<SpriteImage>>>,
    /// The `<CONSTRUCTION>` and `<SCAFFOLD>` entries (item 4163), read on
    /// the first foundation or scaffold drawn, so frame zero pays nothing.
    construction: Option<Vec<ConstructionArt>>,
    scaffolds: Option<Vec<ScaffoldArt>>,
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
    /// The lobby's tileset's `BASELAND` textures (item 4127,
    /// `crate::ground`); `None` where they do not read, and the ground is
    /// drawn in its debug colour.
    ground: Option<GroundDraw>,
    /// Draw the ground in its land kind's debug colour, the row's floor
    /// (item 4127); off by default, `--debug-ground` turns it on.
    pub debug_ground: bool,
    /// The drawn terrain without the textured tiles, the textured tiles
    /// and the transition patch laid over them, at the terrain version
    /// they were built at.
    drawn: (u32, TerrainMesh, Vec<TexturedMesh>, Vec<PatchMesh>),
    /// The screen's ramp the water and the floor take (item 4178), set by
    /// [`World::set_screen`]; `None` lays [`crate::ground::ScreenRamp::MIDDLE`].
    ramp: Option<crate::ground::ScreenRamp>,
    /// The ramp the drawn layers were last laid at, the water's and the
    /// floor shade's places among the patches, and the patches' version.
    laid_ramp: Option<crate::ground::ScreenRamp>,
    screen_layers: (Option<usize>, Option<usize>),
    patch_version: u32,
}

/// The ground's textures, as the draw takes them.
struct GroundDraw {
    textures: crate::ground::GroundTextures,
    /// [`crate::ground::GroundTextures::flat`], shared with each mesh.
    flat: Vec<std::sync::Arc<render::silhouette::Mips>>,
    /// The transition patch's art (item 4139); `None` where the alphas,
    /// the tileset's numbers or the start dump's seed do not read, and no
    /// patch is laid.
    patch: Option<PatchArt>,
    /// A mountain's own art (item 4141); `None` where it does not read,
    /// and the mountains keep the debug grey.
    mountains: Option<MountainDraw>,
}

/// [`crate::ground::MountainArt`] and its textures, as the draw shares them.
struct MountainDraw {
    art: crate::ground::MountainArt,
    plain: std::sync::Arc<render::silhouette::Mips>,
    blend: std::sync::Arc<render::silhouette::Mips>,
    /// Per template: its main and ring alphas.
    alphas: Vec<Option<[std::sync::Arc<render::silhouette::Mips>; 2]>>,
}

/// The key a mountain's stage 0 texture takes beside the base layer's
/// `-1 - index` (item 4141): `MOUNTAIN_PIECE` the main's, one less the
/// ring's.
pub const MOUNTAIN_PIECE: i32 = -100_000;

/// The key a template's alpha takes beside the transition patch's 14
/// (item 4141): `MOUNTAIN_ALPHA + 2 · template`, plus one for the ring.
pub const MOUNTAIN_ALPHA: usize = 1_000;

/// The water's alpha ramp's key among the alphas (item 4164).
pub const WATER_ALPHA: usize = 900;

/// The water's own colour's piece among the ground's textures (item 4178),
/// apart from every base's `-1 - index` and the mountains'.
pub const WATER_PIECE: i32 = -200_000;

/// The ocean floor's baked layers' piece (item 4178), and one less its
/// shade's.
pub const FLOOR_PIECE: i32 = -200_010;

/// What [`crate::ground::our_patches`] and the patch's draw take.
struct PatchArt {
    /// The 14 alphas in name order, each its own level 0.
    alphas: Vec<std::sync::Arc<render::silhouette::Mips>>,
    codes: Vec<[u32; 8]>,
    tileset: String,
    root: PathBuf,
    /// `World.seed`, the texture fractal's (`TerrainData::texture_fractal`).
    seed: u32,
}

/// **The transition patch's draw** (item 4139; `docs/RENDERER.md` §3.9):
/// the patch's base's triangles over one alpha, laid after the base at the
/// alpha's A (`SRCALPHA`, `INVSRCALPHA`); each corner's base texcoord and
/// its texcoord into the alpha, read on level 0, the nearest texel,
/// clamped. Item 4141: or one layer of a mountain's own draw
/// (`docs/RENDERER.md` §3.10), where `mountain` is set.
pub struct PatchMesh {
    /// The patch's base, keyed as the base layer's: `-1 - index` into
    /// [`crate::ground::GroundTextures::flat`].
    pub piece: i32,
    pub positions: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub alpha_uvs: Vec<[f32; 2]>,
    pub mips: std::sync::Arc<render::silhouette::Mips>,
    /// The alpha's index in name order, and its image.
    pub alpha: usize,
    pub alpha_mips: std::sync::Arc<render::silhouette::Mips>,
    /// A mountain's layer (item 4141): `TOP_ADDSIGNED` under the alpha,
    /// A the two texels' product, the alpha linear and clamped.
    pub mountain: bool,
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
    /// [`TreeMeshes::textures`] on their one level, shared with the GPU's
    /// tree pass (item 4173).
    pub leaf_mips: Vec<Option<std::sync::Arc<render::silhouette::Mips>>>,
    /// The section's `Shadow Texture` on its one level (item 4182), the
    /// decal under each tree; `None` where it does not decode.
    pub shadow: Option<std::sync::Arc<render::silhouette::Mips>>,
}

/// **The shadows' art** (item 4182; `docs/RENDERER.md` §3.7, "The
/// shadows"): `effects_graphics.xml`'s `<SHADOW>` textures in file order,
/// each on its one level (`None` where one does not decode), and each unit
/// piece's `shadowwidth`, `shadowheight` and its type's index there.
pub struct ShadowArt {
    pub textures: Vec<Option<std::sync::Arc<render::silhouette::Mips>>>,
    pub units: BTreeMap<i32, (f32, f32, usize)>,
}

/// The key a decal's texture takes beside the figures' pieces (item 4182):
/// `DECAL_PIECE − 1 − k` a `<SHADOW>`'s, `DECAL_PIECE` the trees'.
pub const DECAL_PIECE: i32 = -200_000;

/// One drawn shadow: the decal and the texture key it samples
/// ([`DECAL_PIECE`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    pub decal: render::decal::Decal,
    pub texture: i32,
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

/// One tree texture's faces as corners and texcoords (item 4173).
type TreeFaces = (Vec<[f32; 3]>, Vec<[f32; 2]>);

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
    /// frame zero (the tests' door; the binary names its game).
    #[cfg(test)]
    pub fn open(install_root: &Path, logs: &Path) -> Result<World, String> {
        Self::open_game(install_root, logs, Game::Pair)
    }

    /// Open the install and stand up the game `game` names at frame zero.
    /// The install's tables are loaded once, and only the runner `game`
    /// names is built (item 4161: `--lobby` loaded them twice and built the
    /// pair's runner only to drop it, 0.68 s of its 1.47 s to frame zero).
    /// The ground's art is the pair's start's in both (its tileset and
    /// seed; the lobby is the same map, run53's `game_random seed 12345`).
    pub fn open_game(install_root: &Path, logs: &Path, game: Game) -> Result<World, String> {
        let install = Install::new(install_root);
        if !install.looks_valid() {
            return Err(format!("no install at {}", install_root.display()));
        }
        let loaded = rondata::load::load(&install).map_err(|e| e.to_string())?;
        let start = logs.join(START_DUMP);
        let runner = match game {
            Game::Pair => {
                let long = logs.join(LONG_DUMP);
                let trace = rondata::trace::Trace::read(&logs.join(LONG_TRACE))
                    .map_err(|e| format!("{LONG_TRACE}: {e}"))?;
                FreeRunner::from_captures(
                    &loaded,
                    &long,
                    &[start.as_path()],
                    trace.as_ref(),
                    sim::Tuning::RON,
                )
                .map_err(|e| format!("{}: {e}", long.display()))?
            }
            Game::Lobby => {
                let lobby = logs.join(rondata::script::LOBBY_DUMP);
                FreeRunner::from_lobby(&loaded, &lobby, sim::Tuning::RON)
                    .map_err(|e| format!("{}: {e}", lobby.display()))?
            }
        };
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
        let shadow_files = rondata::artdata::shadow_files(&install);
        let mut shadow_art = ShadowArt {
            textures: shadow_files
                .iter()
                .map(|(_, f)| {
                    let bytes = std::fs::read(resolve_art(install.root(), f)?).ok()?;
                    let image = render::tga::decode(&bytes).ok()?;
                    Some(std::sync::Arc::new(render::silhouette::Mips {
                        levels: vec![image],
                    }))
                })
                .collect(),
            units: BTreeMap::new(),
        };
        for e in unit_art(&install, &graphs) {
            let [ty, w, h] = &e.shadow;
            let shadow = shadow_files
                .iter()
                .position(|(n, _)| n.eq_ignore_ascii_case(ty))
                .zip(w.parse::<f32>().ok())
                .zip(h.parse::<f32>().ok())
                .map(|((k, w), h)| (w, h, k));
            for p in &e.pieces {
                if let Some(sh) = shadow {
                    shadow_art.units.entry(*p).or_insert(sh);
                }
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
        let tileset = lobby_tileset(install_root, &start);
        let seed = rondata::mapgen::start_head(&start)
            .ok()
            .flatten()
            .map(|h| rondata::mapgen::start_grid(&h).seed as u32);
        let ground = tileset.as_deref().and_then(|t| {
            let textures = crate::ground::GroundTextures::load(install.root(), t).ok()?;
            let flat = textures
                .flat()
                .into_iter()
                .map(std::sync::Arc::new)
                .collect();
            let patch = seed.and_then(|seed| {
                let loaded = crate::ground::load_alphas(install.root()).ok()?;
                Some(PatchArt {
                    codes: loaded.iter().map(|a| a.1).collect(),
                    alphas: loaded
                        .into_iter()
                        .map(|a| std::sync::Arc::new(a.0))
                        .collect(),
                    tileset: t.to_string(),
                    root: install.root().to_path_buf(),
                    seed,
                })
            });
            let mountains = crate::ground::MountainArt::load(install.root(), t)
                .ok()
                .map(|art| MountainDraw {
                    plain: std::sync::Arc::new(art.plain.clone()),
                    blend: std::sync::Arc::new(art.blend.clone()),
                    alphas: art
                        .templates
                        .iter()
                        .map(|tp| {
                            tp.as_ref().map(|tp| {
                                [
                                    std::sync::Arc::new(tp.main.clone()),
                                    std::sync::Arc::new(tp.ring.clone()),
                                ]
                            })
                        })
                        .collect(),
                    art,
                });
            Some(GroundDraw {
                textures,
                flat,
                patch,
                mountains,
            })
        });
        let tree_art = tileset
            .and_then(|t| rondata::artdata::tree_art(&install, &t).ok())
            .map(|art| {
                let root = install.root().to_path_buf();
                let textures: Vec<Option<render::tga::Image>> = art
                    .textures
                    .iter()
                    .map(|t| {
                        let bytes = std::fs::read(resolve_art(&root, t)?).ok()?;
                        render::tga::decode(&bytes).ok()
                    })
                    .collect();
                // Item 4173: each texture on its one level, as
                // `RTreeData::tree_textures` holds it (§3.7), for the GPU.
                let leaf_mips = textures
                    .iter()
                    .map(|t| {
                        t.as_ref().map(|i| {
                            std::sync::Arc::new(render::silhouette::Mips {
                                levels: vec![i.clone()],
                            })
                        })
                    })
                    .collect();
                let shadow = art.shadow.as_ref().and_then(|t| {
                    let bytes = std::fs::read(resolve_art(&root, t)?).ok()?;
                    let image = render::tga::decode(&bytes).ok()?;
                    Some(std::sync::Arc::new(render::silhouette::Mips {
                        levels: vec![image],
                    }))
                });
                TreeMeshes {
                    leaf_mips,
                    shadow,
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
                    textures,
                }
            });
        let cats = loaded.unit_types.iter().map(|t| t.cols.cat).collect();
        Ok(World {
            look: BuildLook::new(&install).with_types(&loaded.build_types),
            tribes,
            meshes: BTreeMap::new(),
            unit_models,
            unit_textures,
            shadow_art,
            shadows: true,
            shadow_mirror: false,
            unit_shadows: false,
            figure_textures: BTreeMap::new(),
            textured: true,
            root: install.root().to_path_buf(),
            install,
            pose: true,
            sprites: true,
            sprite_zoom: crate::PINNED_SPRITE_ZOOM,
            sprite_images: BTreeMap::new(),
            construction: None,
            scaffolds: None,
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
            ground,
            debug_ground: false,
            drawn: (u32::MAX, TerrainMesh::default(), Vec::new(), Vec::new()),
            ramp: None,
            laid_ramp: None,
            screen_layers: (None, None),
            patch_version: 0,
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

    /// **The screen the ground is drawn on** (item 4178): `camera` on a
    /// `size` screen, whose [`crate::ground::ScreenRamp`] the water's colour
    /// and the floor's grey follow, as `modify_water` and
    /// `modify_ocean_floor` lay them each frame.
    pub fn set_screen(&mut self, camera: &render::Camera, size: (u32, u32)) {
        self.ramp = Some(crate::ground::ScreenRamp::of(camera, size));
    }

    /// The version of [`World::drawn_terrain`]'s patches (item 4178): it
    /// moves when the screen's ramp lays them again, the terrain unmoved.
    pub fn patch_version(&self) -> u32 {
        self.patch_version
    }

    /// **What the client draws of the terrain** (item 4127): the
    /// heightfield without the `BASELAND` tiles the ground's base layer
    /// textures, and those tiles as one [`TexturedMesh`] per texture, keyed
    /// by the negative piece `-1 - index` into
    /// [`crate::ground::GroundTextures::flat`]. Under
    /// [`World::debug_ground`], or where the textures do not read, the whole
    /// heightfield in its debug colours and no textured tile.
    /// [`World::terrain`] stays the whole heightfield the readers take.
    /// Item 4139: and the transition patch, our own choice on our own
    /// cells, as one [`PatchMesh`] per (patch's base, alpha), drawn after
    /// the base.
    pub fn drawn_terrain(&mut self) -> (&TerrainMesh, &[TexturedMesh], &[PatchMesh]) {
        let version = self.runner.terrain().version;
        if self.debug_ground || self.ground.is_none() {
            return (self.terrain(), &[], &[]);
        }
        // Nothing built yet: no debug tile, no textured mesh and no layer.
        // From item 4164 a map can leave no tile in the debug colours, so
        // the debug mesh alone is not the mark (it rebuilt every frame).
        let unbuilt =
            self.drawn.1.positions.is_empty() && self.drawn.2.is_empty() && self.drawn.3.is_empty();
        if self.drawn.0 != version || unbuilt {
            self.terrain();
            let t = self.runner.terrain();
            let g = self.ground.as_ref().expect("checked above");
            let mountains: std::collections::BTreeSet<(i32, i32)> =
                t.mountains.iter().flatten().map(|p| (p.x, p.y)).collect();
            // Item 4141: with a mountain's own art, the ground is laid
            // beneath it too.
            let none = std::collections::BTreeSet::new();
            let under = if g.mountains.is_some() {
                &none
            } else {
                &mountains
            };
            let base = crate::ground::ground_base(&t, under, &g.textures, &self.terrain.1);
            let textured: std::collections::BTreeSet<(i32, i32)> =
                base.tile.iter().copied().collect();
            let field = Heightfield {
                width: t.width,
                height: t.height,
                corners: &t.corners,
            };
            let placed: Vec<(i32, i32)> = mountains.iter().copied().collect();
            let rest = TerrainMesh::build_except(&field, &t.land, &placed, &|x, y| {
                textured.contains(&(x, y))
            });
            let mut meshes: Vec<TexturedMesh> = g
                .flat
                .iter()
                .enumerate()
                .map(|(k, mips)| TexturedMesh {
                    piece: -1 - k as i32,
                    who: 0,
                    positions: Vec::new(),
                    uvs: Vec::new(),
                    mips: mips.clone(),
                })
                .collect();
            // Item 4178 (§3.12): the land pass draws no `OCEAN` tile;
            // `render_ocean_floor` draws the floor there, its two layers
            // baked at the brightest vertex grey, and a black layer over it
            // takes it to the ramp's grey ([`crate::ground::floor_shade`]).
            let ocean = g.textures.index_land(render::terrain::OCEAN, 0, 0, 0);
            let mut floor = ocean.filter(|&k| k < g.flat.len()).map(|k| {
                let top = crate::ground::floor_grey(0.0);
                let tex = crate::ground::floor_texture(
                    &g.flat[k].levels[0],
                    crate::ground::FLOOR_MIRROR,
                    top,
                );
                TexturedMesh {
                    piece: FLOOR_PIECE,
                    who: 0,
                    positions: Vec::new(),
                    uvs: Vec::new(),
                    mips: std::sync::Arc::new(render::silhouette::Mips::new(tex)),
                }
            });
            let mut shade = PatchMesh {
                piece: FLOOR_PIECE - 1,
                positions: Vec::new(),
                uvs: Vec::new(),
                alpha_uvs: Vec::new(),
                mips: std::sync::Arc::new(render::silhouette::Mips::new(render::tga::Image {
                    width: 1,
                    height: 1,
                    kind: 2,
                    bits: 32,
                    footer: false,
                    rgba: vec![0, 0, 0, 255],
                })),
                alpha: WATER_ALPHA,
                alpha_mips: std::sync::Arc::new(crate::ground::water_ramp()),
                mountain: false,
            };
            for ((tri, uv), &k) in base.tris.iter().zip(&base.uvs).zip(&base.of) {
                if let (Some(f), true) = (floor.as_mut(), Some(k) == ocean) {
                    f.positions.extend_from_slice(tri);
                    f.uvs.extend(tri.iter().map(|p| {
                        let r = crate::ground::FLOOR_REPEAT;
                        [p[0] / r, p[1] / r]
                    }));
                    shade.positions.extend_from_slice(tri);
                    shade.uvs.extend_from_slice(&[[0.5, 0.5]; 3]);
                    continue;
                }
                meshes[k].positions.extend_from_slice(tri);
                meshes[k].uvs.extend_from_slice(uv);
            }
            meshes.extend(floor);
            meshes.retain(|m| !m.positions.is_empty());
            let mut patches: BTreeMap<(usize, usize), PatchMesh> = BTreeMap::new();
            let choice = g.patch.as_ref().and_then(|a| {
                let c = crate::ground::our_patches(&t, &a.root, &a.tileset, a.seed, &a.codes)?;
                Some((a, c))
            });
            if let Some((art, choice)) = choice {
                let laid =
                    crate::ground::ground_patch(&t, under, &g.textures, &self.terrain.1, &choice);
                for i in 0..laid.tris.len() {
                    let (k, a) = (laid.of[i], laid.alpha[i]);
                    let (Some(mips), Some(alpha_mips)) = (g.flat.get(k), art.alphas.get(a)) else {
                        continue;
                    };
                    let m = patches.entry((k, a)).or_insert_with(|| PatchMesh {
                        piece: -1 - k as i32,
                        positions: Vec::new(),
                        uvs: Vec::new(),
                        alpha_uvs: Vec::new(),
                        mips: mips.clone(),
                        alpha: a,
                        alpha_mips: alpha_mips.clone(),
                        mountain: false,
                    });
                    m.positions.extend_from_slice(&laid.tris[i]);
                    m.uvs.extend_from_slice(&laid.uvs[i]);
                    m.alpha_uvs.extend_from_slice(&laid.alpha_uvs[i]);
                }
            }
            let mut patches: Vec<PatchMesh> = patches.into_values().collect();
            let mut layers = (None, None);
            if !shade.positions.is_empty() {
                layers.1 = Some(0);
                patches.insert(0, shade);
            }
            // Item 4164: the water over every cell that holds it, at the
            // water's alpha on a linear ramp so the patch's shader takes it
            // unchanged. Item 4178: on the water's own mesh, flat at z 0,
            // `generate_water`'s alpha, and `modify_water`'s colour, the
            // screen's ramp of `OceanVars`' top bytes ([`World::set_screen`]).
            let water = crate::ground::water_mesh(&t, &crate::ground::water_grid(&t));
            if !water.tris.is_empty() {
                let mut m = PatchMesh {
                    piece: WATER_PIECE,
                    positions: Vec::new(),
                    uvs: Vec::new(),
                    alpha_uvs: Vec::new(),
                    mips: std::sync::Arc::new(crate::ground::water_colour_ramp()),
                    alpha: WATER_ALPHA,
                    alpha_mips: std::sync::Arc::new(crate::ground::water_ramp()),
                    mountain: false,
                };
                for (tri, a) in water.tris.iter().zip(&water.alpha) {
                    m.positions.extend_from_slice(tri);
                    m.uvs.extend_from_slice(&[[0.5, 0.5]; 3]);
                    for a in a {
                        m.alpha_uvs.push([(a + 0.5) / 256.0, 0.5]);
                    }
                }
                layers.0 = Some(patches.len());
                patches.push(m);
            }
            // Item 4141 (§3.10): each template's ring, then its main, after
            // the patch, `render_mountains`'s order.
            if let Some(md) = g.mountains.as_ref() {
                let range = crate::ground::ground_mountain(&t, &md.art, &self.terrain.1);
                let mut layers: BTreeMap<(usize, usize), PatchMesh> = BTreeMap::new();
                for i in 0..range.tris.len() {
                    let k = range.template[i];
                    let Some(Some([main, ring])) = md.alphas.get(k) else {
                        continue;
                    };
                    for (layer, (piece, mips, alpha_mips)) in [
                        (MOUNTAIN_PIECE - 1, &md.blend, ring),
                        (MOUNTAIN_PIECE, &md.plain, main),
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        let m = layers.entry((k, layer)).or_insert_with(|| PatchMesh {
                            piece,
                            positions: Vec::new(),
                            uvs: Vec::new(),
                            alpha_uvs: Vec::new(),
                            mips: mips.clone(),
                            alpha: MOUNTAIN_ALPHA + 2 * k + 1 - layer,
                            alpha_mips: alpha_mips.clone(),
                            mountain: true,
                        });
                        m.positions.extend_from_slice(&range.tris[i]);
                        for px in range.uvs[i] {
                            m.uvs.push([px[0] / 16.0, px[1] / 16.0]);
                            m.alpha_uvs.push(md.art.alpha_uv(k, px));
                        }
                    }
                }
                patches.extend(layers.into_values());
            }
            self.drawn = (version, rest, meshes, patches);
            self.screen_layers = layers;
            self.laid_ramp = None;
        }
        // Item 4178: the water's colour and the floor's grey follow the
        // screen's ramp; only their texcoords are laid again when it moves.
        let ramp = self.ramp.unwrap_or(crate::ground::ScreenRamp::MIDDLE);
        if self.laid_ramp != Some(ramp) {
            let at = |p: &[f32; 3]| ramp.at(p[0], p[1]);
            if let Some(m) = self.screen_layers.0.and_then(|i| self.drawn.3.get_mut(i)) {
                m.uvs = m
                    .positions
                    .iter()
                    .map(|p| crate::ground::ramp_uv(at(p)))
                    .collect();
            }
            if let Some(m) = self.screen_layers.1.and_then(|i| self.drawn.3.get_mut(i)) {
                m.alpha_uvs = m
                    .positions
                    .iter()
                    .map(|p| crate::ground::ramp_uv(crate::ground::floor_shade(at(p)) / 255.0))
                    .collect();
            }
            self.laid_ramp = Some(ramp);
            self.patch_version = self.patch_version.wrapping_add(1);
        }
        (&self.drawn.1, &self.drawn.2, &self.drawn.3)
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
                    normals: Vec::new(),
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

    /// **The trees as the client draws them** (item 4173; §3.7, "The trees'
    /// own colour"): each face that samples a texture into that texture's
    /// mesh — `piece` the texture's index, `who` [`TREE_WHO`] — for
    /// `client::gpu`'s tree pass, which binds the leaf's texel as
    /// `client::trees::DRAWN` colours it; a face with no texcoords or no
    /// decoded texture into the window's soup, flat green as before.
    pub fn tree_textured(&self, soup: &mut Soup) -> Vec<TexturedMesh> {
        let Some(art) = self.tree_art.as_ref() else {
            return Vec::new();
        };
        let mut by: BTreeMap<usize, TreeFaces> = BTreeMap::new();
        let cuts = self.tree_cuts();
        for (d, cut) in self.tree_drawings().iter().zip(&cuts) {
            for (ti, tri) in d.tris.iter().enumerate() {
                let textured = cut.as_ref().and_then(|c| {
                    let k = *c.image.get(ti)?;
                    art.leaf_mips.get(k)?.as_ref()?;
                    Some((k, *c.uvs.get(ti)?))
                });
                match textured {
                    Some((k, uv)) => {
                        let e = by.entry(k).or_default();
                        e.0.extend(tri);
                        e.1.extend(uv);
                    }
                    None => {
                        let lit = shade(*tri);
                        for p in tri {
                            soup.positions.push(*p);
                            soup.colours.push(crate::trees::FLAT_GREEN.map(|v| v * lit));
                        }
                    }
                }
            }
        }
        by.into_iter()
            .filter_map(|(k, (positions, uvs))| {
                Some(TexturedMesh {
                    piece: k as i32,
                    who: TREE_WHO,
                    positions,
                    uvs,
                    mips: art.leaf_mips.get(k)?.clone()?,
                })
            })
            .collect()
    }

    /// **The frame's shadows** (item 4182; `docs/RENDERER.md` §3.7, "The
    /// shadows"): `ObjectsOut::sort_trees`' decal at every tree
    /// [`World::tree_list`] stands, and `GuyOut::draw_shadows`' under every
    /// figure of `list` whose piece names a `<SHADOW>`, at its point with no
    /// `shadow_offset` (the packet's are 0 to 11 units on 31 of run1559b's
    /// 32 guys; not ported) and turned against its drawn degree. Empty with
    /// [`World::shadows`] off.
    pub fn shadows(&self, figures: &[Placed], list: &[Drawn]) -> Vec<Shadow> {
        if !self.shadows {
            return Vec::new();
        }
        self.shadows_of(figures, list, self.unit_shadows)
    }

    /// The frame's decals whether or not they are drawn, the guys' in or
    /// out as `units` says: the row selects with them in.
    pub fn shadows_of(&self, figures: &[Placed], list: &[Drawn], units: bool) -> Vec<Shadow> {
        let mut out = Vec::new();
        if self.trees && self.tree_art.as_ref().is_some_and(|a| a.shadow.is_some()) {
            for t in self.tree_list() {
                out.push(Shadow {
                    decal: render::decal::tree_shadow([t.at.0, t.at.1, t.at.2]),
                    texture: DECAL_PIECE,
                });
            }
        }
        let at_figure: BTreeMap<(u8, i16, u16), &Placed> = figures
            .iter()
            .map(|p| ((p.who, p.o, p.figure), p))
            .collect();
        for d in list.iter().filter(|_| units) {
            let Pose::Figure { figure, facing, .. } = d.pose else {
                continue;
            };
            let Some(p) = at_figure.get(&(d.who, d.o, figure)) else {
                continue;
            };
            let Some(&(w, h, k)) = self.shadow_art.units.get(&d.piece) else {
                continue;
            };
            let drawn = sim::data::pivot::release_rotation(sim::movement::Angle(facing as i32));
            out.push(Shadow {
                decal: render::decal::unit_shadow([p.x, p.y, p.z], w, h, drawn),
                texture: DECAL_PIECE - 1 - k as i32,
            });
        }
        if self.shadow_mirror {
            for s in &mut out {
                s.decal.xoffset = -s.decal.xoffset;
            }
        }
        out
    }

    /// A shadow's texture by its key ([`DECAL_PIECE`]).
    pub fn shadow_texture(&self, key: i32) -> Option<&std::sync::Arc<render::silhouette::Mips>> {
        if key == DECAL_PIECE {
            return self.tree_art.as_ref()?.shadow.as_ref();
        }
        let k = usize::try_from(DECAL_PIECE - 1 - key).ok()?;
        self.shadow_art.textures.get(k)?.as_ref()
    }

    /// [`World::shadows`] as the GPU's decal pass takes them: one mesh a
    /// texture, `piece` its key, in the order the shadows fall.
    pub fn shadow_meshes(&self, shadows: &[Shadow]) -> Vec<TexturedMesh> {
        type Faces = (i32, Vec<[f32; 3]>, Vec<[f32; 2]>);
        let mut by: Vec<Faces> = Vec::new();
        for s in shadows {
            let (p, uv) = s.decal.triangles();
            match by.iter_mut().find(|e| e.0 == s.texture) {
                Some(e) => {
                    e.1.extend(p);
                    e.2.extend(uv);
                }
                None => by.push((s.texture, p.to_vec(), uv.to_vec())),
            }
        }
        by.into_iter()
            .filter_map(|(piece, positions, uvs)| {
                Some(TexturedMesh {
                    piece,
                    who: 0,
                    positions,
                    uvs,
                    mips: self.shadow_texture(piece)?.clone(),
                })
            })
            .collect()
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
        let (model, scale, files) = if let Some(e) = keyed(piece, SCAFFOLD_KEY) {
            let s = self.scaffold_art().get(e)?;
            (s.model.clone(), SCAFFOLD_SCALE, Vec::new())
        } else if building {
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
        let name = match keyed(piece, SCAFFOLD_KEY) {
            Some(e) => self.scaffold_art().get(e).map(|s| s.texture.clone()),
            None => self.unit_textures.get(&piece).cloned(),
        };
        let decoded = name
            .and_then(|t| resolve_art(&self.root, &t))
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

    /// Item 4117's alternative: `piece` drawn with the install's file
    /// `name` (an XML path, `.\art\x.tga`) in place of its own, or its own
    /// again with `None`. `false` when the file does not resolve or decode.
    #[cfg(test)]
    pub fn swap_figure_texture(&mut self, piece: i32, name: Option<&str>) -> bool {
        self.figure_textures.remove(&piece);
        let Some(name) = name else { return true };
        let decoded = resolve_art(&self.root, name)
            .and_then(|p| std::fs::read(p).ok())
            .and_then(|b| render::tga::decode(&b).ok())
            .map(|i| std::sync::Arc::new(render::silhouette::Mips::new(i)));
        let ok = decoded.is_some();
        self.figure_textures.insert(piece, decoded);
        ok
    }

    /// A building piece's sprite, decoded once.
    pub fn sprite_image(&mut self, piece: i32) -> Option<std::sync::Arc<SpriteImage>> {
        if let Some(s) = self.sprite_images.get(&piece) {
            return s.clone();
        }
        let (texture, offset) = match keyed(piece, FOUNDATION_KEY) {
            Some(slot) => {
                let c = self.construction_art().get(slot);
                (
                    c.map(|c| c.texture.clone()),
                    c.map_or((0, 0), |c| c.sprite_offset),
                )
            }
            None => {
                let set = std::iter::once(piece).collect();
                let art = build_art(&self.install, &set);
                let a = art.get(&piece).and_then(|a| a.as_ref().ok());
                (
                    a.map(|a| a.texture.clone()),
                    a.map_or((0, 0), |a| a.sprite_offset),
                )
            }
        };
        let decoded = texture.and_then(|texture| {
            let bytes = std::fs::read(resolve_art(&self.root, &texture)?).ok()?;
            let image = render::tga::decode(&bytes).ok()?;
            let covered = image
                .rgba
                .as_chunks::<4>()
                .0
                .iter()
                .map(|p| p[3] >= 128)
                .collect();
            Some(std::sync::Arc::new(SpriteImage {
                image,
                offset,
                covered,
            }))
        });
        self.sprite_images.insert(piece, decoded.clone());
        decoded
    }

    /// The `<CONSTRUCTION>` entries, read once (item 4163).
    fn construction_art(&mut self) -> &[ConstructionArt] {
        self.construction
            .get_or_insert_with(|| construction_art(&self.install))
    }

    /// The `<SCAFFOLD>` entries, read once (item 4163).
    fn scaffold_art(&mut self) -> &[ScaffoldArt] {
        self.scaffolds
            .get_or_insert_with(|| scaffold_art(&self.install))
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
            // A foundation is its `<CONSTRUCTION>` slot's sprite at its own
            // anchor (item 4163): `draw_build`, as a building's.
            let piece = match d.pose {
                Pose::Building { .. } => {
                    let Some(b) = at_building.get(&(d.who, d.o)) else {
                        continue;
                    };
                    if !self.sprite_drawn(d.piece, b.type_index) {
                        continue;
                    }
                    d.piece
                }
                Pose::Foundation { slot, .. } if self.sprites => foundation_key(slot),
                _ => continue,
            };
            let Some(image) = self.sprite_image(piece) else {
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
                piece,
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
            let mut scaffold = None;
            let mut piece = d.piece;
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
                // The foundation is a `<CONSTRUCTION>` sprite
                // ([`World::sprite_drawings`], item 4163).
                Pose::Foundation { .. } => continue,
                // A scaffold piece is its `<SCAFFOLD>` element's model at
                // the slot's point and angle (item 4163), part of the
                // building: no figure's silhouette.
                Pose::Scaffold {
                    index,
                    element,
                    angle,
                    ..
                } => {
                    let Some(b) = at_building.get(&(d.who, d.o)) else {
                        continue;
                    };
                    let Some(p) = b
                        .scaffold(self.look.scaffold_timing())
                        .into_iter()
                        .find(|p| p.index == index)
                    else {
                        continue;
                    };
                    scaffold = Some(angle);
                    piece = scaffold_key(element);
                    let (x, y, z) = p.point;
                    ([x, y, z], 0, true, None)
                }
            };
            let Some(mesh) = self.mesh(piece, building) else {
                continue;
            };
            let m = if let Some(angle) = scaffold {
                guy_placement(point, angle, mesh.scale)
            } else if building {
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
            // Each corner's `id 4` texcoord, one a vertex (item 4099); a
            // scaffold's too (item 4163).
            let uvs = if (building && scaffold.is_none())
                || mesh.model.texcoords.len() != mesh.model.verts.len()
            {
                Vec::new()
            } else {
                mesh.faces
                    .iter()
                    .map(|f| f.map(|i| mesh.model.texcoords[i as usize]))
                    .collect()
            };
            // The stored normals through the same map (item 4122): each
            // vertex moved by its normal, posed and placed, less the vertex
            // posed and placed — the linear part alone.
            let normals =
                if !cfg!(test) || building || mesh.model.normals.len() != mesh.model.verts.len() {
                    Vec::new()
                } else {
                    let mut tipped = mesh.model.clone();
                    for (v, n) in tipped.verts.iter_mut().zip(&mesh.model.normals) {
                        *v = [v[0] + n[0], v[1] + n[1], v[2] + n[2]];
                    }
                    let tips = match clock
                        .filter(|_| self.pose)
                        .and_then(|(slot, t)| mesh.anims.get(&slot).map(|a| (a, t)))
                    {
                        Some((a, t)) => posed(&tipped, a, anim_ms(t, a)),
                        None => rest_pose(&tipped),
                    };
                    let at: Vec<[f32; 3]> = verts
                        .iter()
                        .zip(&tips)
                        .map(|(v, t)| {
                            let (a, b) = (apply(&m, *v), apply(&m, *t));
                            let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                            let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                            if l > 0.0 && l.is_finite() {
                                d.map(|x| x / l)
                            } else {
                                [0.0; 3]
                            }
                        })
                        .collect();
                    mesh.faces
                        .iter()
                        .map(|f| f.map(|i| at[i as usize]))
                        .collect()
                };
            out.push(Drawing {
                who: d.who,
                o: d.o,
                piece,
                building,
                tris,
                uvs,
                normals,
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
                    who: d.who,
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
/// the owner's colour through the texel's alpha, and
/// `render::silhouette::paint` samples.
pub struct TexturedMesh {
    pub piece: i32,
    /// The owner, whose unit colour the texel's alpha lets through (4102).
    pub who: u8,
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
    /// Each corner's stored `id 3` normal (its first three words) carried
    /// to world space by the corner's own node, pose and placement — the
    /// linear part of the map `tris` takes, nothing else — and
    /// renormalised (item 4122); empty for a building or a tree, where
    /// the model's normals are not one a vertex, and outside the tests:
    /// read by no draw, only by `silhouette::tests::the_stored_normals`.
    #[cfg_attr(not(test), allow(dead_code))]
    pub normals: Vec<[[f32; 3]; 3]>,
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
pub(crate) fn shade(t: [[f32; 3]; 3]) -> f32 {
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

    /// The window capture beside the pair (`rondata::diff::aztec_modern_pair`'s
    /// `RUN1429B`): run1429's frames 3295 to 3389, every one printing its
    /// buildings.
    const RUN1429B: &str = "gamelog-run1429b-greatlakes-aztec-modern-window-3295-3389.txt";

    /// **The foundations the client draws, of the draw list's** (item 4163:
    /// new, 877 of 877): on the 99 (dump, frame) pairs below, each
    /// `Pose::Foundation` entry drawn as its `<CONSTRUCTION>` slot's decoded
    /// sprite. The list's 877 are
    /// `rondata::free::tests::FOUNDATIONS_IN_REGISTER`'s, drawn by ours.
    const CLIENT_FOUNDATIONS_DRAWN: (usize, usize) = (877, 877);

    /// **The scaffold pieces the client draws, of the draw list's** (item
    /// 4163: new, 2,781 of 2,781): each `Pose::Scaffold` entry drawn as its
    /// `<SCAFFOLD>` element's decoded model, every triangle placed. The
    /// list's 2,781 are `SCAFFOLDS_IN_REGISTER`'s, drawn by ours.
    const CLIENT_SCAFFOLDS_DRAWN: (usize, usize) = (2781, 2781);

    /// **The client draws the foundation and the scaffold, each with its own
    /// piece** (item 4163, `docs/RENDERER.md` §3.5). On every frame a dump of
    /// the lobby prints buildings on — run1428's two, run1429's start and
    /// 3390, run1429b's 95: the (dump, frame) pairs on which
    /// `rondata::free::tests::the_draw_list_s_buildings_are_in_register`
    /// holds the list's foundations and scaffolds in register — the client's
    /// game is stepped to the frame and its draw list's entries of the two
    /// poses handed to the client's own draws: a foundation is drawn when
    /// [`World::sprite_drawings`] gives it its slot's sprite at
    /// `render::draw::sprite_at` of its anchor, a scaffold piece when
    /// [`World::objects`] gives it its element's model, placed by
    /// `guy_placement` at the slot's point and angle at
    /// [`super::SCAFFOLD_SCALE`]. Made to fail: the `<CONSTRUCTION>` read
    /// one slot on draws the eleventh slot's entries with no sprite.
    /// A `<CONSTRUCTION>` sprite's width, height and offset.
    type OwnSprite = (usize, usize, (i32, i32));

    #[test]
    fn the_client_draws_each_foundation_and_scaffold_with_its_own_piece() {
        use super::{SCAFFOLD_SCALE, foundation_key, scaffold_key};
        use render::draw::Pose;
        let install = std::env::var_os("RON_INSTALL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("../../game"));
        let logs = gamelog_dir();
        let names = [super::START_DUMP, super::LONG_DUMP, RUN1429B];
        if let Some(n) = names.iter().find(|n| !logs.join(n).exists()) {
            println!("skipping: no {n}");
            return;
        }
        let mut world = match World::open(&install, &logs) {
            Ok(w) => w,
            Err(e) => {
                println!("skipping: {e}");
                return;
            }
        };
        // (frame, dump) for every frame a dump prints a building on, as the
        // register test reads them: a record with its type and piece.
        let printed = |b: &[rondata::gamelog::BuildDump]| {
            b.iter()
                .any(|b| b.orig_type.is_some() && b.gpiece.is_some())
        };
        let mut wanted: Vec<(i64, usize)> = Vec::new();
        for (i, name) in names.iter().enumerate() {
            let mut src = rondata::capture::indexed::IndexedCapture::open(logs.join(name)).unwrap();
            let setup = src.read_replay_setup().unwrap();
            let log = rondata::gamelog::Log::parse(&setup);
            if log.initial().is_some_and(|init| printed(&init.builds)) {
                wanted.push((0, i));
            }
            for f in src.frame_states() {
                let f = f.unwrap();
                if printed(&f.builds) {
                    wanted.push((f.n, i));
                }
            }
        }
        wanted.sort();
        assert_eq!(wanted.len(), 99, "the register test's (dump, frame) pairs");
        // Each entry's own file, read apart from the client: a
        // `<CONSTRUCTION>` sprite's size and offset, a `<SCAFFOLD>` model's
        // triangles.
        let inst = rondata::Install::new(&install);
        let read = |rel: &str| {
            rondata::artdata::resolve_art(inst.root(), rel).and_then(|p| std::fs::read(p).ok())
        };
        let own_sprites: Vec<Option<OwnSprite>> = rondata::artdata::construction_art(&inst)
            .iter()
            .map(|c| {
                let i = render::tga::decode(&read(&c.texture)?).ok()?;
                Some((i.width, i.height, c.sprite_offset))
            })
            .collect();
        let own_models: Vec<Option<usize>> = rondata::artdata::scaffold_art(&inst)
            .iter()
            .map(|s| Some(render::bh3::decode(&read(&s.model)?).ok()?.faces.len()))
            .collect();
        let (camera, screen) = (crate::pinned_camera(), crate::pinned_screen());
        // frame → (foundations listed, drawn, scaffolds listed, drawn).
        let mut by_frame: std::collections::BTreeMap<i64, [usize; 4]> = Default::default();
        let mut sample = None;
        for &(n, _) in &wanted {
            if by_frame.contains_key(&n) {
                continue;
            }
            while world.runner.frame() < n {
                world.runner.step();
            }
            assert_eq!(world.runner.scene(None).frame, n);
            let (_, buildings, list) = world.draw_list(&camera, &screen, (1920, 1080));
            let of = |found: bool| -> Vec<render::draw::Drawn> {
                list.iter()
                    .filter(|d| match d.pose {
                        Pose::Foundation { .. } => found,
                        Pose::Scaffold { .. } => !found,
                        _ => false,
                    })
                    .copied()
                    .collect()
            };
            let (founds, scaffs) = (of(true), of(false));
            let sprites = world.sprite_drawings(&buildings, &founds, camera.zoom_distance);
            let drawn_founds = sprites
                .iter()
                .zip(&founds)
                .filter(|(s, d)| {
                    let Pose::Foundation { slot, .. } = d.pose else {
                        return false;
                    };
                    let at = render::draw::sprite_at(
                        d.anchor,
                        s.image.offset,
                        world.sprite_zoom.0,
                        world.sprite_zoom.1,
                        camera.zoom_distance,
                    );
                    let own = (s.image.image.width, s.image.image.height, s.image.offset);
                    (s.who, s.o, s.piece, s.at) == (d.who, d.o, foundation_key(slot), at)
                        && own_sprites.get(usize::from(slot)) == Some(&Some(own))
                })
                .count();
            let drawings = world.objects(&[], &buildings, &scaffs);
            let drawn_scaffs = drawings
                .iter()
                .zip(&scaffs)
                .filter(|(m, d)| {
                    let Pose::Scaffold { element, .. } = d.pose else {
                        return false;
                    };
                    (m.who, m.o, m.piece) == (d.who, d.o, scaffold_key(element))
                        && own_models.get(usize::from(element)) == Some(&Some(m.tris.len()))
                        && m.building
                        && !m.tris.is_empty()
                        && m.tris.iter().flatten().flatten().all(|v| v.is_finite())
                })
                .count();
            if sample.is_none()
                && let (Some(s), Some(d), Some(m), Some(e)) = (
                    sprites.first(),
                    founds.first(),
                    drawings.first(),
                    scaffs.first(),
                )
            {
                sample = Some(format!(
                    "frame {n}: foundation {}/{} {:?} anchor ({}, {}) sprite {:?} {}x{}; \
                     scaffold {}/{} {:?} anchor ({}, {}) piece {} {} triangles",
                    d.who,
                    d.o,
                    d.pose,
                    d.anchor.x,
                    d.anchor.y,
                    s.at,
                    s.image.image.width,
                    s.image.image.height,
                    e.who,
                    e.o,
                    e.pose,
                    e.anchor.x,
                    e.anchor.y,
                    m.piece,
                    m.tris.len()
                ));
            }
            by_frame.insert(n, [founds.len(), drawn_founds, scaffs.len(), drawn_scaffs]);
        }
        let mut t = [0usize; 4];
        for (n, _) in &wanted {
            for (a, b) in t.iter_mut().zip(by_frame[n]) {
                *a += b;
            }
        }
        println!(
            "foundations {} of {}, scaffolds {} of {}; {}",
            t[1],
            t[0],
            t[3],
            t[2],
            sample.unwrap_or_default()
        );
        assert_eq!(SCAFFOLD_SCALE, 3.0);
        assert_eq!((t[1], t[0]), CLIENT_FOUNDATIONS_DRAWN, "foundations drawn");
        assert_eq!((t[3], t[2]), CLIENT_SCAFFOLDS_DRAWN, "scaffolds drawn");
    }
}
