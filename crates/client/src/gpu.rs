//! The GPU half (`wgpu`, earned here: `docs/RENDERER.md` §4.2, §3.6): one
//! pipeline for the world's triangles under the original's matrix, one for
//! the overlay in screen space, drawn to a window's surface or to a texture
//! read back for a still.
//!
//! **The projection is the camera's own.** The vertex shader multiplies by
//! `render::Camera::to_screen_matrix` uploaded row-major, which WGSL reads
//! as its transpose, so `M * v` is the original's row-vector `v · M`; the
//! divide by `w` and the viewport's `((x + 1)·W/2, (1 − y)·H/2)` are the
//! ones `MiscAccess::world_to_console` makes. Only depth is the client's:
//! the camera-space z (`render_matrix`'s third column, the draw list's
//! depth key) over a fixed range, since the original's near and far planes
//! would clip the map's far corners at this camera.

use std::sync::Arc;

// Item 4181: each shader is a `.wgsl` file under `crates/client/shaders/`,
// read here as it stands — the strings these constants held, character for
// character, a leading newline and all — and each parsed and validated by
// `naga` in `every_shader_parses_and_validates` below.

/// **The world's and the overlay's shader**: the world's triangles under
/// the camera's matrix and depth, the overlay in screen space, each corner
/// its colour.
const SHADER: &str = include_str!("../shaders/world.wgsl");

/// **The sprites' shader** (item 4076): a building's `texture=` as a quad in
/// screen pixels at `render::draw::SpriteAt`, at the draw list's depth for
/// its anchor, a pixel kept where its alpha is half or more. It is blitted
/// over the ground (depth compare always) and writes the anchor's depth, so
/// a figure nearer than the anchor draws over it and one further does not — the pixels
/// `render::silhouette::Stamp` counts. No team colour (`build_sprite_color`)
/// is applied.
const SPRITE_SHADER: &str = include_str!("../shaders/sprite.wgsl");

/// **The figures' textured shader** (item 4099): the world pipeline's
/// projection and depth, line for line, with each corner's `id 4` texcoord
/// in place of a colour, and the `<UNIT>`'s `texture=` sampled unlit —
/// bilinear within a level, the level nearest, wrapped, `v = 0` the image's
/// top row — the arithmetic `render::silhouette::paint` runs on the CPU
/// for the colour reader (`docs/RENDERER.md` §5.3). The card picks the
/// level per pixel and the CPU per triangle. Each corner carries its
/// owner's unit colour, laid through the texel's alpha as
/// `render::silhouette::team_blend` lays it (item 4102, §5.3 "The team
/// colour"): `t·α + c·(1 − α)`. `fs_silhouette` is the silhouette's pass
/// (item 4103, §5.3 "The silhouette's colour"): the owner's colour at
/// `render::silhouette::SILHOUETTE_ALPHA`, blended source-alpha where the
/// figure is behind what is drawn, as `render::silhouette::
/// silhouette_blend` lays it. `fs_ground` is the ground's base layer
/// (item 4127, `docs/RENDERER.md` §3.6): the texel's colour alone, under
/// `determine_land_height_color`'s white, the alpha (225 to 255 on the
/// files) not laid over anything.
const TEXTURED_SHADER: &str = include_str!("../shaders/textured.wgsl");

/// [`TEXTURED_SHADER`] with its holes filled: the units' material for
/// `fs_textured_lit` (item 4345), `UNIT_DIFFUSE` and `UNIT_AMBIENT ·
/// UNIT_AMBIENT_I`.
fn textured_shader() -> String {
    use render::silhouette::{UNIT_AMBIENT, UNIT_AMBIENT_I, UNIT_DIFFUSE};
    TEXTURED_SHADER
        .replace("{UNIT_DIFFUSE}", &format!("{UNIT_DIFFUSE:?}"))
        .replace(
            "{UNIT_AMBIENT_R}",
            &format!("{:?}", UNIT_AMBIENT[0] * UNIT_AMBIENT_I),
        )
        .replace(
            "{UNIT_AMBIENT_G}",
            &format!("{:?}", UNIT_AMBIENT[1] * UNIT_AMBIENT_I),
        )
        .replace(
            "{UNIT_AMBIENT_B}",
            &format!("{:?}", UNIT_AMBIENT[2] * UNIT_AMBIENT_I),
        )
}

/// **The trees' pass** (item 4173; `docs/RENDERER.md` §3.7, "The trees'
/// own colour"), appended to [`TEXTURED_SHADER`]: the leaf's texture on
/// its one level, bilinear and clamped, cut where its alpha is not above
/// `render_states[9]`'s 0; then `ADDSIGNED` with the vertex diffuse,
/// `TL_FAKE_LIGHT`'s power and HDR scale, and the tone map at the clamp —
/// `render::silhouette::fake_light` on `tree_stage`, the arithmetic the
/// painter runs. On the plain `Unorm` target and texture (item 4171) the
/// sample is the byte and the result is stored as it stands.
///
/// `shaders/tree.wgsl` is the template: each `{NAME}` in it is the
/// constant of that name, written as `format!("{NAME:?}")` writes it — the
/// literal the shader carried before item 4181 moved it.
const TREE_SHADER: &str = include_str!("../shaders/tree.wgsl");

/// The names [`TREE_SHADER`] leaves a hole for, each filled once.
#[cfg(test)]
const TREE_HOLES: [&str; 5] = [
    "current",
    "FAKE_LIGHT_POWER",
    "HDR_SCALE",
    "EXPOSURE",
    "white",
];

/// [`TREE_SHADER`] with its holes filled.
fn tree_shader() -> String {
    use render::silhouette::{
        EXPOSURE, FAKE_LIGHT_POWER, HDR_SCALE, MIN_LUMINANCE, TREE_DIFFUSE, WHITE_POINT, filmic,
    };
    let white = 1.0 / filmic(MIN_LUMINANCE * WHITE_POINT);
    let current = f32::from(TREE_DIFFUSE) / 255.0;
    TREE_SHADER
        .replace("{current}", &format!("{current:?}"))
        .replace("{FAKE_LIGHT_POWER}", &format!("{FAKE_LIGHT_POWER:?}"))
        .replace("{HDR_SCALE}", &format!("{HDR_SCALE:?}"))
        .replace("{EXPOSURE}", &format!("{EXPOSURE:?}"))
        .replace("{white}", &format!("{white:?}"))
}

/// **The transition patch's pass** (item 4139; `docs/RENDERER.md` §3.9),
/// `TerrainOut::render`'s `showtrans` block as its `RenderState` reads:
/// stage 0 the patch's base land texture, under the vertex white
/// (MODULATE with DIFFUSE, the texel alone); stage 1 the alpha, its A
/// taken (MODULATE with TEXTURE and CURRENT, the diffuse alpha 1), read on
/// level 0, `MIP_POINT` and `TEXADD_CLAMP` — so a nearest, clamped sampler
/// on the alpha's own level 0; blended `SRCALPHA`, `INVSRCALPHA` over the
/// base already drawn. The vertex is the textured layout's: the alpha's
/// texcoord rides in the third attribute's `xy`.
const PATCH_SHADER: &str = include_str!("../shaders/patch.wgsl");
/// **The shadows' pass** (item 4182; `docs/RENDERER.md` §3.7, "The
/// shadows"): the decal's black under its texel's alpha, after the textured
/// shader's vertex stage.
const DECAL_SHADER: &str = include_str!("../shaders/decal.wgsl");
/// **The shadows' share taken through the tone map** (item 4182): the
/// ground under the decals composed with the keep map, `T(T⁻¹(g)·keep)`.
const SHADE_SHADER: &str = include_str!("../shaders/shade.wgsl");
/// The keep map's format: the product of every decal's `1 − a`.
const KEEP_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R16Float;

/// [`SHADE_SHADER`] with the tone mapper's live constants (§3.14).
fn shade_shader() -> String {
    use render::silhouette::{EXPOSURE, MIN_LUMINANCE, WHITE_POINT, filmic};
    let white = 1.0 / filmic(MIN_LUMINANCE * WHITE_POINT);
    SHADE_SHADER
        .replace("{EXPOSURE}", &format!("{EXPOSURE:?}"))
        .replace("{white}", &format!("{white:?}"))
}

/// **The land's lit branch** (item 4212; `docs/RENDERER.md` §3.21),
/// appended to [`TEXTURED_SHADER`] and [`PATCH_SHADER`]: `land_lit`, which
/// `fs_ground`, `fs_patch` and `fs_mountain` take their stage colour
/// through.
const LAND_SHADER: &str = include_str!("../shaders/land.wgsl");

/// The names [`LAND_SHADER`] leaves a hole for, each filled once.
#[cfg(test)]
const LAND_HOLES: [&str; 13] = [
    "LIT_POWER",
    "FAKE_LIGHT_POWER",
    "HDR_SCALE",
    "LAND_DIFFUSE",
    "N_DOT_L",
    "AMBIENT_R",
    "AMBIENT_G",
    "AMBIENT_B",
    "VAMBIENT_R",
    "VAMBIENT_G",
    "VAMBIENT_B",
    "EXPOSURE",
    "white",
];

/// [`LAND_SHADER`] with its holes filled.
fn land_shader() -> String {
    use render::silhouette::{
        EXPOSURE, FAKE_LIGHT_POWER, HDR_SCALE, LAND_AMBIENT, LAND_DIFFUSE, LAND_N_DOT_L, LIT_POWER,
        MIN_LUMINANCE, SCENE_AMBIENT, WHITE_POINT, filmic,
    };
    let white = 1.0 / filmic(MIN_LUMINANCE * WHITE_POINT);
    LAND_SHADER
        .replace("{LIT_POWER}", &format!("{LIT_POWER:?}"))
        .replace("{FAKE_LIGHT_POWER}", &format!("{FAKE_LIGHT_POWER:?}"))
        .replace("{HDR_SCALE}", &format!("{HDR_SCALE:?}"))
        .replace("{LAND_DIFFUSE}", &format!("{LAND_DIFFUSE:?}"))
        .replace("{N_DOT_L}", &format!("{LAND_N_DOT_L:?}"))
        .replace("{AMBIENT_R}", &format!("{:?}", SCENE_AMBIENT[0]))
        .replace("{AMBIENT_G}", &format!("{:?}", SCENE_AMBIENT[1]))
        .replace("{AMBIENT_B}", &format!("{:?}", SCENE_AMBIENT[2]))
        .replace("{VAMBIENT_R}", &format!("{:?}", LAND_AMBIENT[0]))
        .replace("{VAMBIENT_G}", &format!("{:?}", LAND_AMBIENT[1]))
        .replace("{VAMBIENT_B}", &format!("{:?}", LAND_AMBIENT[2]))
        .replace("{EXPOSURE}", &format!("{EXPOSURE:?}"))
        .replace("{white}", &format!("{white:?}"))
}

/// **The lake lit and blended in HDR** (item 4232; `docs/RENDERER.md`
/// §3.24), appended to [`PATCH_SHADER`] after [`LAND_SHADER`]: `fs_floor`,
/// the floor's stage per pixel through the lit branch, and `fs_water`, the
/// water lit at its material and blended over the frame under it in HDR.
const WATER_SHADER: &str = include_str!("../shaders/water.wgsl");

/// **The particles' shader** (item 4233; `docs/RENDERER.md` §3.25): a
/// quad facing the camera, the smoke's `MODULATE2X` and the fire's
/// `MODULATE`, each taken through `TL_FAKE_LIGHT` into HDR (item 4237).
const PARTICLE_SHADER: &str = include_str!("../shaders/particle.wgsl");

/// The names [`PARTICLE_SHADER`] leaves a hole for, each filled once.
#[cfg(test)]
const PARTICLE_HOLES: [&str; 2] = ["FAKE_LIGHT_POWER", "HDR_SCALE"];

/// [`PARTICLE_SHADER`] with its holes filled.
fn particle_shader() -> String {
    use render::silhouette::{FAKE_LIGHT_POWER, HDR_SCALE};
    PARTICLE_SHADER
        .replace("{FAKE_LIGHT_POWER}", &format!("{FAKE_LIGHT_POWER:?}"))
        .replace("{HDR_SCALE}", &format!("{HDR_SCALE:?}"))
}

// **The plume target** (item 4237; `docs/RENDERER.md` §3.25): the light
// the particles lay in, in rgb, and the share of the frame under them
// left, in alpha, each in HDR; item 4241 keeps it on the expose frame.
use crate::expose::PLUME_FORMAT;

/// The names [`WATER_SHADER`] leaves a hole for, each filled once.
#[cfg(test)]
const WATER_HOLES: [&str; 12] = [
    "WATER_POWER",
    "WATER_R",
    "WATER_G",
    "WATER_B",
    "WATER_N_DOT_L",
    "WATER_AMBIENT",
    "SCENE_R",
    "SCENE_G",
    "SCENE_B",
    "WATER_HDR",
    "SURF_POWER",
    "SURF_HDR",
];

/// [`WATER_SHADER`] with its holes filled.
fn water_shader() -> String {
    use crate::ground::{WATER_AMBIENT, WATER_DIFFUSE};
    use render::silhouette::{FAKE_LIGHT_POWER, HDR_SCALE, LAND_N_DOT_L, LIT_POWER, SCENE_AMBIENT};
    WATER_SHADER
        .replace("{WATER_POWER}", &format!("{LIT_POWER:?}"))
        .replace("{WATER_R}", &format!("{:?}", WATER_DIFFUSE[0]))
        .replace("{WATER_G}", &format!("{:?}", WATER_DIFFUSE[1]))
        .replace("{WATER_B}", &format!("{:?}", WATER_DIFFUSE[2]))
        .replace("{WATER_N_DOT_L}", &format!("{LAND_N_DOT_L:?}"))
        .replace("{WATER_AMBIENT}", &format!("{WATER_AMBIENT:?}"))
        .replace("{SCENE_R}", &format!("{:?}", SCENE_AMBIENT[0]))
        .replace("{SCENE_G}", &format!("{:?}", SCENE_AMBIENT[1]))
        .replace("{SCENE_B}", &format!("{:?}", SCENE_AMBIENT[2]))
        .replace("{WATER_HDR}", &format!("{HDR_SCALE:?}"))
        .replace("{SURF_POWER}", &format!("{FAKE_LIGHT_POWER:?}"))
        .replace("{SURF_HDR}", &format!("{HDR_SCALE:?}"))
}

/// The names [`SHADE_SHADER`] leaves a hole for, each filled once.
#[cfg(test)]
const SHADE_HOLES: [&str; 2] = ["EXPOSURE", "white"];

/// **Every shader module `Gpu::new` builds**, as its label and its source
/// assembled as the device takes it: the one list `Gpu::new` compiles and
/// `every_shader_parses_and_validates` validates, so a module cannot be
/// built without being validated. In `Gpu::new`'s order: the world's, the
/// sprites', the textured, the decal (the textured with
/// [`DECAL_SHADER`] appended), the tree (the textured with the filled
/// [`TREE_SHADER`] appended), the shade, the patch, the luminance and the
/// expose (item 4212).
fn shader_sources() -> [(&'static str, String); 11] {
    [
        ("client", SHADER.to_string()),
        ("sprites", SPRITE_SHADER.to_string()),
        (
            "textured",
            format!("{}{}", textured_shader(), land_shader()),
        ),
        (
            "decal",
            format!("{}{}{DECAL_SHADER}", textured_shader(), land_shader()),
        ),
        (
            "tree",
            format!("{}{}{}", textured_shader(), land_shader(), tree_shader()),
        ),
        ("shade", shade_shader()),
        (
            "patch",
            format!("{PATCH_SHADER}{}{}", land_shader(), water_shader()),
        ),
        ("luminance", crate::expose::luminance_shader()),
        ("expose", crate::expose::expose_shader()),
        ("particle", particle_shader()),
        ("bloom", crate::expose::BLOOM_SHADER.to_string()),
    ]
}

/// The camera-space depth range mapped onto [0, 1]: wider than any map.
const DEPTH_RANGE: (f32, f32) = (-200_000.0, 200_000.0);

/// **The underlay's alpha** (item 4401; `docs/HUD.md` §3):
/// `WallOut::draw_underlays@006410a0` clears the top bit of the owner's
/// `TeamColor::get_unit` alpha unless `Camera::is_zoom_out`, so `0xff`
/// draws at `0x7f`. That `get_unit`'s alpha is `0xff` is not read.
pub const UNDERLAY_ALPHA: u8 = 0x7f;

/// Depth, and a stencil the silhouette pass marks its pixels in (item 4171).
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24PlusStencil8;
/// The frame's clear colour, off the heightfield.
const CLEAR: wgpu::Color = wgpu::Color {
    r: 0.01,
    g: 0.01,
    b: 0.015,
    a: 1.0,
};

/// **The silhouette's pixel is blended once** (item 4171; `docs/RENDERER.md`
/// §5.3, "The silhouette's colour"; §3.15): `c·a + n·(1 − a)` over
/// the nearer pixel, as `render::silhouette::silhouette_blend` lays it. A
/// closed mesh behind a tree has a near and a far side, and both pass
/// depth Greater; the stencil keeps the first and refuses the rest, where
/// two blends would lay `0.75·c + 0.25·n`.
const ONCE: wgpu::StencilFaceState = wgpu::StencilFaceState {
    compare: wgpu::CompareFunction::Equal,
    fail_op: wgpu::StencilOperation::Keep,
    depth_fail_op: wgpu::StencilOperation::Keep,
    pass_op: wgpu::StencilOperation::IncrementClamp,
};

/// Triangles as positions and colours, three vertices each.
pub type Tris<'a> = (&'a [[f32; 3]], &'a [[f32; 3]]);

/// **One layer of the screen pass** (item 4410; `docs/HUD.md` §3): quads
/// in normalised device coordinates, every one at `alpha`, the pass's
/// blend constant, `c·a + d·(1 − a)`. The interface's textures carry a
/// handful of alphas (the scroll's 255 and 224, the star's edge), so a
/// layer of each draws them in one call; layers draw in order, so a later
/// layer lies over an earlier one.
#[derive(Clone, Debug, Default)]
pub struct ScreenLayer {
    pub alpha: u8,
    pub pos: Vec<[f32; 3]>,
    pub col: Vec<[f32; 3]>,
}

/// One frame's triangles: the terrain's (kept on the GPU while its version
/// stands), the figures' and the overlay's.
pub struct Frame<'a> {
    pub terrain_version: u32,
    pub terrain: Tris<'a>,
    pub figures: Tris<'a>,
    pub overlay: Tris<'a>,
    /// The underlays (items 4401, 4421): the selected buildings' bands,
    /// drawn after the ground and before the objects, each layer blended
    /// at its alpha (the texel's) times [`UNDERLAY_ALPHA`].
    pub underlay: &'a [ScreenLayer],
    /// The screen pass (item 4410): the cities' plates, after the tone map
    /// and before the overlay, each layer blended at its alpha.
    pub screen: &'a [ScreenLayer],
    /// The figures drawn textured (item 4099).
    pub textured: &'a [crate::world::TexturedMesh],
    /// The trees' textured faces, one mesh a texture (item 4173).
    pub trees: &'a [crate::world::TexturedMesh],
    /// The ground's base layer (item 4127), kept on the GPU with the
    /// terrain at `terrain_version`.
    pub ground: &'a [crate::world::TexturedMesh],
    /// The transition patch over the base (item 4139), kept with it.
    pub patch: &'a [crate::world::PatchMesh],
    /// The patches' version (item 4178): uploaded again when it moves,
    /// the terrain's version standing.
    pub patch_version: u32,
    /// The surf's version (item 4239): when only it moves, the surf's
    /// lists alone are uploaded again.
    pub surf_version: u32,
    /// Item 4327: the land pass's layers blended on an HDR target
    /// ([`crate::world::World::land_hdr`]).
    pub land_hdr: bool,
    /// Item 4338: the surf's lists blended on the HDR target
    /// ([`crate::world::World::surf_hdr`]).
    pub surf_hdr: bool,
    /// Item 4379: the national borders blended on the HDR target
    /// ([`crate::world::World::border_hdr`]).
    pub border_hdr: bool,
    /// Item 4344: the objects before the water and the surf
    /// ([`crate::world::World::objects_under_water`]).
    pub objects_before_water: bool,
    /// The buildings drawn as their sprites (item 4076).
    pub sprites: &'a [crate::world::SpriteDrawing],
    /// The shadows (item 4182), one mesh a decal texture.
    pub shadows: &'a [crate::world::TexturedMesh],
    /// The burning buildings' particles (item 4233), one mesh a blend.
    pub particles: &'a [crate::particle::ParticleMesh],
}

/// A ground pass over `view`, cleared or loaded (item 4232).
fn begin<'e>(
    enc: &'e mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    depth_view: &wgpu::TextureView,
    clear: bool,
) -> wgpu::RenderPass<'e> {
    let (colour, depth, stencil) = if clear {
        (
            wgpu::LoadOp::Clear(CLEAR),
            wgpu::LoadOp::Clear(1.0),
            wgpu::LoadOp::Clear(0),
        )
    } else {
        (wgpu::LoadOp::Load, wgpu::LoadOp::Load, wgpu::LoadOp::Load)
    };
    enc.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("ground"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: colour,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth_view,
            depth_ops: Some(wgpu::Operations {
                load: depth,
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: Some(wgpu::Operations {
                load: stencil,
                store: wgpu::StoreOp::Store,
            }),
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    })
}

/// **Where the figures' silhouette pass falls** (item 4171; `docs/RENDERER.md`
/// §5.3, "The silhouettes"; §3.15).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SilhouetteOrder {
    /// Every figure's silhouette (`render_states[1]`, depth Greater, no
    /// depth write) before any figure's lit draw: the depth it tests is the
    /// ground's, the sprites' and the trees', as the original's list 1
    /// draws `[1]` and then `[0]` (4085).
    Before,
    /// The draw before item 4171, kept as the rows' made-to-fail: after the lit figures, whose own depth
    /// is then written, so a closed mesh's far side passes on its own near
    /// side and the figure is washed in its owner's colour.
    #[cfg_attr(not(test), allow(dead_code))]
    After,
}

/// A mountain main layer's vertex under its maps (item 4278): the patch's
/// eight singles, then the corner's normal and tangent.
const MAPPED_STRIDE: u64 = 56;

/// A mesh on the GPU: its vertices, their count and its texture.
type Uploaded = (wgpu::Buffer, u32, wgpu::BindGroup);

/// A patch mesh on the GPU: [`Uploaded`], its alpha's bind group,
/// whether it is a mountain's layer (item 4141), and how it is shaded.
type UploadedPatch = (
    wgpu::Buffer,
    u32,
    wgpu::BindGroup,
    wgpu::BindGroup,
    bool,
    PatchKind,
    Option<wgpu::BindGroup>,
);

/// **How a patch mesh is shaded** (items 4212, 4232).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PatchKind {
    /// Its colour as it stands: the roads and the weathering.
    Plain,
    /// The land's light (item 4212; `docs/RENDERER.md` §3.21): a land
    /// texture's layer (keyed `-1 − index`, the faded land and the
    /// transition patches) and the coral, which the card draws
    /// `ENABLE_LIGHTING`.
    Lit,
    /// A land texture's faded base layer (item 4310; `docs/RENDERER.md`
    /// §3.27, "The faded land in HDR"), keyed [`crate::world::LAND_FADE_ALPHA`]:
    /// `T(L · a)`, opaque, as the card's target blends it over black.
    Faded,
    /// The ocean floor's stage under its vertex grey, lit (item 4232).
    Floor,
    /// The water, lit and blended over the frame under it in HDR (item
    /// 4232): its draw is a pass of its own, after the frame is copied.
    Water,
    /// The surf (item 4239, §3.27): stage 0's texel, its alpha times the
    /// vertex's and the mask's, over the frame.
    Surf,
    /// The snowcap underlay (item 4275, §3.10): the texel times the land's
    /// vertex colour through `TL_FAKE_LIGHT`, at the texture's alpha.
    Snowcap,
    /// The map's edge border (item 4311, §3.15): the texel through
    /// `TL_FAKE_LIGHT`, opaque (`ONE`, `ZERO`), its depth written.
    Edge,
    /// A good's billboard (item 4312, §3.24): the texel through
    /// `TL_FAKE_LIGHT` under a white vertex, at its own alpha.
    Billboard,
    /// A national border (item 4376, §3.29, "The band's heights"): its
    /// colour as it stands, with the depth test off and its depth
    /// unwritten, as `Scene::init_render_states@008ba450` sets the card for
    /// `Borders::process_all`, so the ground between two of the strip's
    /// samples never covers it.
    Border,
}

/// **The land pass's HDR target** (item 4327; `docs/RENDERER.md` §3.27,
/// "The patches over the faded land in HDR"): the card's own target holds
/// the land's layers before the tone map, and its blends run there.
const LAND_HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

fn patch_kind(piece: i32, alpha: usize) -> PatchKind {
    use crate::world::{CORAL_PIECE, FLOOR_PIECE, LAND_FADE_ALPHA, SNOWCAP_PIECE, WATER_PIECE};
    match piece {
        p if (WATER_PIECE + 1..0).contains(&p) && alpha == LAND_FADE_ALPHA => PatchKind::Faded,
        WATER_PIECE => PatchKind::Water,
        SNOWCAP_PIECE => PatchKind::Snowcap,
        // Item 4319: `render_weathering` draws classes 2 and 3 as it draws
        // class 4.
        p if crate::world::is_land_oil(p) => PatchKind::Snowcap,
        crate::world::OIL_PIECE => PatchKind::Billboard,
        p if crate::world::is_edge(p) => PatchKind::Edge,
        p if crate::world::is_border(p) => PatchKind::Border,
        FLOOR_PIECE => PatchKind::Floor,
        p if crate::world::is_surf(p) => PatchKind::Surf,
        p if (WATER_PIECE + 1..0).contains(&p) || p == CORAL_PIECE => PatchKind::Lit,
        _ => PatchKind::Plain,
    }
}

/// The ground's base layer and its patch on the GPU, at a terrain version.
/// Item 4178: and the patches' own version, which the screen's ramp moves.
/// Item 4239: and the surf's version.
type GroundUploads = (u32, Vec<Uploaded>, Vec<UploadedPatch>, u32, u32);

pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    world: wgpu::RenderPipeline,
    overlay: wgpu::RenderPipeline,
    /// The underlays (item 4401; `docs/HUD.md` §3): world triangles over
    /// the ground and under the buildings, blended at [`UNDERLAY_ALPHA`],
    /// no depth test, no depth written.
    underlay: wgpu::RenderPipeline,
    /// The screen pass (item 4410): the overlay's shader under the
    /// underlay's constant blend, one draw a [`ScreenLayer`].
    screen: wgpu::RenderPipeline,
    uniform: wgpu::Buffer,
    bind: wgpu::BindGroup,
    format: wgpu::TextureFormat,
    terrain: std::sync::Mutex<Option<(u32, wgpu::Buffer, u32)>>,
    /// The ground's base layer (item 4127) and the terrain version it was
    /// uploaded at.
    ground: wgpu::RenderPipeline,
    ground_meshes: std::sync::Mutex<Option<GroundUploads>>,
    /// The transition patch's pass (item 4139), its alphas' nearest,
    /// clamped sampler and each alpha on the GPU, uploaded once.
    patch: wgpu::RenderPipeline,
    /// The national borders' pass (item 4376): `patch` with the depth test
    /// off.
    border: wgpu::RenderPipeline,
    /// The land's patches, lit (item 4212).
    patch_lit: wgpu::RenderPipeline,
    faded: wgpu::RenderPipeline,
    /// Item 4327 (`docs/RENDERER.md` §3.27, "The patches over the faded
    /// land in HDR"): the faded land, the floor, the transition patches and
    /// the coral on the land pass's HDR target ([`LAND_HDR_FORMAT`]), and the
    /// two full-screen passes that take the frame there and back.
    hdr_lit: wgpu::RenderPipeline,
    hdr_faded: wgpu::RenderPipeline,
    hdr_floor: wgpu::RenderPipeline,
    /// Item 4338: the surf on the HDR target (`fs_surf_hdr`).
    hdr_surf: wgpu::RenderPipeline,
    /// Item 4379: the national borders over a copy of the frame, blended
    /// as on the HDR target (`fs_border_over`), the depth test off as
    /// `border`'s.
    border_over: wgpu::RenderPipeline,
    untone: wgpu::RenderPipeline,
    retone: wgpu::RenderPipeline,
    /// The ocean floor, lit per pixel, depth written; the water, blended in
    /// HDR over the frame under it, and that copy's layout (item 4232).
    floor: wgpu::RenderPipeline,
    water: wgpu::RenderPipeline,
    /// The surf's lists (item 4239, §3.27).
    surf: wgpu::RenderPipeline,
    lake_layout: wgpu::BindGroupLayout,
    alpha_sampler: wgpu::Sampler,
    /// A mountain's layers (item 4141, §3.10): their pass, and the alphas'
    /// linear, clamped sampler (`MIP_LINEAR`, `TEXADD_CLAMP`).
    mountain: wgpu::RenderPipeline,
    mountain_sampler: wgpu::Sampler,
    /// A mountain's main layer under its `_mtrl` maps (item 4278, §3.22):
    /// its pass and stage 2's layout (bindings 2 and 3 of group 3).
    mountain_mapped: wgpu::RenderPipeline,
    mtrl_layout: wgpu::BindGroupLayout,
    /// The snowcap underlay's pass (item 4275, §3.10).
    snowcap: wgpu::RenderPipeline,
    billboard: wgpu::RenderPipeline,
    /// The map's edge border's pass (item 4311, §3.15).
    edge: wgpu::RenderPipeline,
    alpha_textures: std::sync::Mutex<std::collections::BTreeMap<usize, wgpu::BindGroup>>,
    /// `mountain_mtrl`'s bind group (item 4278), uploaded on first use.
    mtrl_textures: std::sync::Mutex<Option<wgpu::BindGroup>>,
    sprite: wgpu::RenderPipeline,
    sprite_uniform: wgpu::Buffer,
    sprite_bind: wgpu::BindGroup,
    sprite_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    /// Each building piece's sprite on the GPU, uploaded once.
    textures: std::sync::Mutex<std::collections::BTreeMap<(i32, u8), wgpu::BindGroup>>,
    /// The figures' textured pipeline and its sampler (item 4099).
    textured: wgpu::RenderPipeline,
    /// A lit figure's (item 4345, `fs_textured_lit`).
    textured_lit: wgpu::RenderPipeline,
    /// The figures' silhouette pass (item 4103): `render_states[1]`'s
    /// depth Greater, no depth write, source-alpha blend.
    silhouette: wgpu::RenderPipeline,
    figure_sampler: wgpu::Sampler,
    /// Item 4338: the surf's stage 0, point on every filter and wrapped, as
    /// the card's draws 163–169 set it (`RenderState +0xec`, `+0xf4`, `+0xfc`).
    surf_sampler: wgpu::Sampler,
    /// Each unit piece's texture and its levels on the GPU, uploaded once.
    figure_textures: std::sync::Mutex<std::collections::BTreeMap<i32, wgpu::BindGroup>>,
    /// The trees' pass (item 4173): `render_states[9]`'s LessEqual with
    /// depth written, its linear, clamped sampler, and each tree texture on
    /// the GPU, uploaded once.
    tree: wgpu::RenderPipeline,
    tree_sampler: wgpu::Sampler,
    tree_textures: std::sync::Mutex<std::collections::BTreeMap<i32, wgpu::BindGroup>>,
    /// The shadows' pass (item 4182) and each decal texture on the GPU,
    /// uploaded once under the trees' linear, clamped sampler.
    decal: wgpu::RenderPipeline,
    /// The shade pass (item 4182) and its two textures' layout.
    shade: wgpu::RenderPipeline,
    /// The tone map's white from the frame's mean (item 4212).
    expose: crate::expose::Expose,
    shade_layout: wgpu::BindGroupLayout,
    decal_textures: std::sync::Mutex<std::collections::BTreeMap<i32, wgpu::BindGroup>>,
    /// Where the silhouette pass falls (item 4171).
    pub silhouette_order: SilhouetteOrder,
    /// The particles' passes (item 4233): the smoke laid over, the fire
    /// added; their texture under the trees' linear, clamped sampler. Item
    /// 4237: each into the plume target, in HDR, and `plume` composes it
    /// onto the frame through the tone map.
    smoke: wgpu::RenderPipeline,
    fire: wgpu::RenderPipeline,
    plume: wgpu::RenderPipeline,
    particle_textures: std::sync::Mutex<std::collections::BTreeMap<i32, wgpu::BindGroup>>,
}

/// Block on a future wgpu answers at once on a native backend.
pub fn block_on<F: std::future::Future>(f: F) -> F::Output {
    let mut f = std::pin::pin!(f);
    let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
    loop {
        if let std::task::Poll::Ready(v) = f.as_mut().poll(&mut cx) {
            return v;
        }
        std::thread::yield_now();
    }
}

impl Gpu {
    pub fn new(
        instance: &wgpu::Instance,
        surface: Option<&wgpu::Surface<'_>>,
        format: wgpu::TextureFormat,
    ) -> Result<Gpu, String> {
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: surface,
            apply_limit_buckets: false,
        }))
        .map_err(|e| format!("no adapter: {e}"))?;
        let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("client"),
            ..Default::default()
        }))
        .map_err(|e| format!("no device: {e}"))?;
        // Every module, from the one list the tests validate.
        let [
            module,
            sprite_module,
            textured_module,
            decal_module,
            tree_module,
            shade_module,
            patch_module,
            luminance_module,
            expose_module,
            particle_module,
            bloom_module,
        ] = shader_sources().map(|(label, source)| {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            })
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera"),
            size: 112,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("client"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });
        let attributes = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];
        let vertex_layout = [Some(wgpu::VertexBufferLayout {
            array_stride: 24,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &attributes,
        })];
        let pipeline = |entry: &str, depth: bool, blend: Option<wgpu::BlendState>| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(entry),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    buffers: &vertex_layout,
                },
                primitive: wgpu::PrimitiveState {
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(depth),
                    depth_compare: Some(if depth {
                        wgpu::CompareFunction::Less
                    } else {
                        wgpu::CompareFunction::Always
                    }),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some("fs"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        // Item 4401: the underlay's blend, `c·a + d·(1 − a)` with `a` the
        // pass's constant ([`UNDERLAY_ALPHA`]).
        let constant = wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::Constant,
            dst_factor: wgpu::BlendFactor::OneMinusConstant,
            operation: wgpu::BlendOperation::Add,
        };
        let underlay_blend = Some(wgpu::BlendState {
            color: constant,
            alpha: constant,
        });
        let sprite_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("screen"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let sprite_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("screen"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: sprite_uniform.as_entire_binding(),
            }],
        });
        let sprite_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let sprite_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("sprites"),
                bind_group_layouts: &[Some(&bgl), Some(&sprite_layout)],
                immediate_size: 0,
            });
        let sprite_attributes = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];
        let sprite = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sprites"),
            layout: Some(&sprite_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &sprite_module,
                entry_point: Some("vs_sprite"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: 20,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &sprite_attributes,
                })],
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &sprite_module,
                entry_point: Some("fs_sprite"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sprite"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let textured_attributes =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3];
        let pass_of = |module: &wgpu::ShaderModule,
                       label: &str,
                       entry: &str,
                       write: bool,
                       compare: wgpu::CompareFunction,
                       blend: Option<wgpu::BlendState>,
                       stencil: wgpu::StencilState| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&sprite_pipeline_layout),
                vertex: wgpu::VertexState {
                    module,
                    entry_point: Some("vs_textured"),
                    compilation_options: Default::default(),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: 32,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &textured_attributes,
                    })],
                },
                primitive: wgpu::PrimitiveState {
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(write),
                    depth_compare: Some(compare),
                    stencil: stencil.clone(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module,
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let figure_pass = |label: &str,
                           entry: &str,
                           write: bool,
                           compare: wgpu::CompareFunction,
                           blend: Option<wgpu::BlendState>,
                           stencil: wgpu::StencilState| {
            pass_of(
                &textured_module,
                label,
                entry,
                write,
                compare,
                blend,
                stencil,
            )
        };
        // Item 4182: `enables` 0x20084 on draws 116–119 — no depth test, no
        // depth write, the alpha test and source alpha over the rest: each
        // decal multiplies the keep map by `1 − a`.
        let multiply = wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::Zero,
            dst_factor: wgpu::BlendFactor::Src,
            operation: wgpu::BlendOperation::Add,
        };
        let decal = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("decal"),
            layout: Some(&sprite_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &decal_module,
                entry_point: Some("vs_textured"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: 32,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &textured_attributes,
                })],
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &decal_module,
                entry_point: Some("fs_decal"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: KEEP_FORMAT,
                    blend: Some(wgpu::BlendState {
                        color: multiply,
                        alpha: multiply,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let shade_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shade"),
            entries: &[0, 1].map(|binding| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }),
        });
        let shade_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("shade"),
                bind_group_layouts: &[Some(&shade_layout)],
                immediate_size: 0,
            });
        let shade = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shade"),
            layout: Some(&shade_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shade_module,
                entry_point: Some("vs_full"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shade_module,
                entry_point: Some("fs_shade"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let tree = pass_of(
            &tree_module,
            "tree",
            "fs_tree",
            true,
            wgpu::CompareFunction::LessEqual,
            Some(wgpu::BlendState::ALPHA_BLENDING),
            Default::default(),
        );
        let textured = figure_pass(
            "textured",
            "fs_textured",
            true,
            wgpu::CompareFunction::Less,
            None,
            Default::default(),
        );
        let textured_lit = figure_pass(
            "textured lit",
            "fs_textured_lit",
            true,
            wgpu::CompareFunction::Less,
            None,
            Default::default(),
        );
        // Item 4233 (§3.25): both systems' `enables` 0x20085 on the packets:
        // the depth tested (`zfunc` LessEqual), not written, and `src_blend`
        // 5 (`SRCALPHA`); the smoke's `dst_blend` 6 (`INVSRCALPHA`), the
        // fire's 2 (`ONE`). Item 4237: into the plume target, cleared to
        // no light and the whole frame left. The light blends as the
        // original's HDR target does; the alpha keeps what is left of the
        // frame under them, `τ · (1 − a)` for the smoke and `τ` for the
        // fire, so the compose's `T⁻¹(g) · τ + L` is the draws in order.
        let particle_attributes =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x4];
        let particle_pass = |label: &str, entry: &str, dst: wgpu::BlendFactor| {
            let c = wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: dst,
                operation: wgpu::BlendOperation::Add,
            };
            let left = wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: dst,
                operation: wgpu::BlendOperation::Add,
            };
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&sprite_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &particle_module,
                    entry_point: Some("vs_particle"),
                    compilation_options: Default::default(),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: 36,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &particle_attributes,
                    })],
                },
                primitive: wgpu::PrimitiveState {
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(false),
                    depth_compare: Some(wgpu::CompareFunction::LessEqual),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &particle_module,
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: PLUME_FORMAT,
                        blend: Some(wgpu::BlendState {
                            color: c,
                            alpha: left,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let smoke = particle_pass("smoke", "fs_smoke", wgpu::BlendFactor::OneMinusSrcAlpha);
        let fire = particle_pass("fire", "fs_fire", wgpu::BlendFactor::One);
        // Item 4237: the plume target composed onto the frame, `T(T⁻¹(g) ·
        // τ + L)`, after the shade pass's layout: the frame copied, the
        // plume target.
        let plume = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("plume"),
            layout: Some(&shade_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shade_module,
                entry_point: Some("vs_full"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shade_module,
                entry_point: Some("fs_plume"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let silhouette = figure_pass(
            "silhouette",
            "fs_silhouette",
            false,
            wgpu::CompareFunction::Greater,
            Some(wgpu::BlendState::ALPHA_BLENDING),
            wgpu::StencilState {
                front: ONCE,
                back: ONCE,
                read_mask: 0xff,
                write_mask: 0xff,
            },
        );
        let ground = figure_pass(
            "ground",
            "fs_ground",
            true,
            wgpu::CompareFunction::Less,
            None,
            Default::default(),
        );
        let patch_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("patch"),
            bind_group_layouts: &[Some(&bgl), Some(&sprite_layout), Some(&sprite_layout)],
            immediate_size: 0,
        });
        let lake_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("lake"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });
        let water_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("water"),
            bind_group_layouts: &[
                Some(&bgl),
                Some(&sprite_layout),
                Some(&sprite_layout),
                Some(&lake_layout),
            ],
            immediate_size: 0,
        });
        let patch_pass_to = |label: &str,
                             entry: &str,
                             layout: &wgpu::PipelineLayout,
                             depth_write: bool,
                             format: wgpu::TextureFormat,
                             compare: wgpu::CompareFunction| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module: &patch_module,
                    entry_point: Some("vs_patch"),
                    compilation_options: Default::default(),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: 32,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &textured_attributes,
                    })],
                },
                primitive: wgpu::PrimitiveState {
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(depth_write),
                    depth_compare: Some(compare),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &patch_module,
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let patch_pass = |label: &str, entry: &str, layout: &wgpu::PipelineLayout, depth_write| {
            patch_pass_to(
                label,
                entry,
                layout,
                depth_write,
                format,
                wgpu::CompareFunction::LessEqual,
            )
        };
        let blended = |label: &str, entry: &str| patch_pass(label, entry, &patch_layout, false);
        // Item 4327: the land pass's layers on its HDR target, as `faded`,
        // `floor` and `patch_lit` are on the frame.
        let hdr_lit = patch_pass_to(
            "hdr_lit",
            "fs_patch_lit_hdr",
            &patch_layout,
            false,
            LAND_HDR_FORMAT,
            wgpu::CompareFunction::LessEqual,
        );
        let hdr_faded = patch_pass_to(
            "hdr_faded",
            "fs_faded_hdr",
            &patch_layout,
            false,
            LAND_HDR_FORMAT,
            wgpu::CompareFunction::LessEqual,
        );
        let hdr_floor = patch_pass_to(
            "hdr_floor",
            "fs_floor_hdr",
            &patch_layout,
            true,
            LAND_HDR_FORMAT,
            wgpu::CompareFunction::LessEqual,
        );
        // Item 4338: the surf's lists on the same target.
        let hdr_surf = patch_pass_to(
            "hdr_surf",
            "fs_surf_hdr",
            &patch_layout,
            false,
            LAND_HDR_FORMAT,
            wgpu::CompareFunction::LessEqual,
        );
        // Item 4379: the national borders through `TL_FAKE_LIGHT`, over a
        // copy of the frame, with the depth test off (item 4376).
        let border_over = patch_pass_to(
            "border_over",
            "fs_border_over",
            &water_layout,
            false,
            format,
            wgpu::CompareFunction::Always,
        );
        let full_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("full"),
            bind_group_layouts: &[None, None, None, Some(&lake_layout)],
            immediate_size: 0,
        });
        let full = |label: &str, entry: &str, format: wgpu::TextureFormat, write| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&full_layout),
                vertex: wgpu::VertexState {
                    module: &patch_module,
                    entry_point: Some("vs_full"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &patch_module,
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: write,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let untone = full(
            "untone",
            "fs_untone",
            LAND_HDR_FORMAT,
            wgpu::ColorWrites::ALL,
        );
        // The frame's alpha is left as the passes before laid it.
        let retone = full("retone", "fs_retone", format, wgpu::ColorWrites::COLOR);
        // Item 4278: stage 2's `_mtrl` map, group 3's bindings 2 and 3.
        let mtrl_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mtrl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let mapped_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mountain_mapped"),
            bind_group_layouts: &[
                Some(&bgl),
                Some(&sprite_layout),
                Some(&sprite_layout),
                Some(&mtrl_layout),
            ],
            immediate_size: 0,
        });
        let mapped_attributes = wgpu::vertex_attr_array![
            0 => Float32x3, 1 => Float32x2, 2 => Float32x3, 3 => Float32x3, 4 => Float32x3
        ];
        let mountain_mapped = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("mountain_mapped"),
            layout: Some(&mapped_layout),
            vertex: wgpu::VertexState {
                module: &patch_module,
                entry_point: Some("vs_mountain_mapped"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: MAPPED_STRIDE,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &mapped_attributes,
                })],
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &patch_module,
                entry_point: Some("fs_mountain_mapped"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let patch = blended("patch", "fs_patch");
        let border = patch_pass_to(
            "border",
            "fs_patch",
            &patch_layout,
            false,
            format,
            wgpu::CompareFunction::Always,
        );
        let patch_lit = blended("patch_lit", "fs_patch_lit");
        // Item 4310: opaque by its alpha of 1, its depth unwritten as the
        // faded layer's was.
        let faded = blended("faded", "fs_faded");
        let mountain = blended("mountain", "fs_mountain");
        let snowcap = blended("snowcap", "fs_snowcap");
        let billboard = blended("billboard", "fs_billboard");
        // Item 4311: opaque, so its alpha of 1 replaces; its depth written.
        let edge = patch_pass("edge", "fs_edge", &patch_layout, true);
        let surf = patch_pass("surf", "fs_surf", &water_layout, false);
        // Item 4232: the floor writes its depth, as the base layer it
        // replaced did; the water reads the frame under it.
        let floor = patch_pass("floor", "fs_floor", &patch_layout, true);
        let water = patch_pass("water", "fs_water", &water_layout, false);
        let alpha_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("alpha"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let mountain_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("mountain"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let surf_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("surf"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let figure_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("figure"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let tree_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("tree"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let expose = crate::expose::Expose::new(
            &device,
            &luminance_module,
            &expose_module,
            &bloom_module,
            format,
            DEPTH_FORMAT,
        );
        Ok(Gpu {
            expose,
            decal,
            shade,
            shade_layout,
            decal_textures: std::sync::Mutex::new(Default::default()),
            tree,
            tree_sampler,
            tree_textures: std::sync::Mutex::new(Default::default()),
            ground,
            ground_meshes: std::sync::Mutex::new(None),
            patch,
            patch_lit,
            faded,
            hdr_lit,
            hdr_faded,
            hdr_floor,
            hdr_surf,
            border_over,
            untone,
            retone,
            floor,
            water,
            surf,
            lake_layout,
            alpha_sampler,
            mountain,
            mountain_sampler,
            mountain_mapped,
            mtrl_layout,
            snowcap,
            billboard,
            edge,
            border,
            alpha_textures: std::sync::Mutex::new(Default::default()),
            mtrl_textures: std::sync::Mutex::new(None),
            textured,
            textured_lit,
            silhouette,
            figure_sampler,
            figure_textures: std::sync::Mutex::new(Default::default()),
            surf_sampler,
            silhouette_order: SilhouetteOrder::Before,
            smoke,
            fire,
            plume,
            particle_textures: std::sync::Mutex::new(Default::default()),
            sprite,
            sprite_uniform,
            sprite_bind,
            sprite_layout,
            sampler,
            textures: std::sync::Mutex::new(Default::default()),
            world: pipeline("vs_world", true, None),
            overlay: pipeline("vs_overlay", false, None),
            underlay: pipeline("vs_world", false, underlay_blend),
            screen: pipeline("vs_overlay", false, underlay_blend),
            device,
            queue,
            uniform,
            bind,
            format,
            terrain: std::sync::Mutex::new(None),
        })
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    /// **The textures' format follows the target's** (item 4171;
    /// `docs/RENDERER.md` §3.15). On a plain `Unorm` target a
    /// texel is sampled, filtered, blended and written as its bytes over
    /// 255, the arithmetic of the original's fixed-function stages and of
    /// `render::silhouette`'s painter; an `…Srgb` target, the draw before
    /// 4171, decodes each texel to linear and encodes the result, so every
    /// blend, `ADDSIGNED` and vertex colour is taken in linear light.
    fn texel_format(&self) -> wgpu::TextureFormat {
        if self.format.is_srgb() {
            wgpu::TextureFormat::Rgba8UnormSrgb
        } else {
            wgpu::TextureFormat::Rgba8Unorm
        }
    }

    /// A building piece's sprite as a bind group, uploaded on first use.
    fn texture(&self, piece: (i32, u8), image: &render::tga::Image) -> wgpu::BindGroup {
        let mut cache = self.textures.lock().unwrap();
        if let Some(b) = cache.get(&piece) {
            return b.clone();
        }
        let extent = wgpu::Extent3d {
            width: image.width as u32,
            height: image.height as u32,
            depth_or_array_layers: 1,
        };
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("sprite"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.texel_format(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.queue.write_texture(
            tex.as_image_copy(),
            &image.rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(image.width as u32 * 4),
                rows_per_image: Some(image.height as u32),
            },
            extent,
        );
        let view = tex.create_view(&Default::default());
        let b = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sprite"),
            layout: &self.sprite_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        cache.insert(piece, b.clone());
        b
    }

    /// A unit piece's texture with its levels as a bind group, uploaded on
    /// first use (item 4099).
    fn figure_texture(&self, piece: i32, mips: &render::silhouette::Mips) -> wgpu::BindGroup {
        let mut cache = self.figure_textures.lock().unwrap();
        if let Some(b) = cache.get(&piece) {
            return b.clone();
        }
        // Item 4338: the surf's slots under the card's point sampler.
        let sampler = if crate::world::is_surf(piece) {
            &self.surf_sampler
        } else {
            &self.figure_sampler
        };
        let b = self.upload_mips(&mips.levels, sampler);
        cache.insert(piece, b.clone());
        b
    }

    /// A transition alpha's level 0 as a bind group under the nearest,
    /// clamped sampler, uploaded on first use (item 4139): the pass reads
    /// level 0 alone (§3.9).
    fn alpha_texture(
        &self,
        alpha: usize,
        mips: &render::silhouette::Mips,
        linear: bool,
    ) -> wgpu::BindGroup {
        let mut cache = self.alpha_textures.lock().unwrap();
        if let Some(b) = cache.get(&alpha) {
            return b.clone();
        }
        let sampler = if linear {
            &self.mountain_sampler
        } else {
            &self.alpha_sampler
        };
        let b = self.upload_mips(&mips.levels[..1], sampler);
        cache.insert(alpha, b.clone());
        b
    }

    /// Levels as one texture beside a sampler, a bind group of the
    /// figures' layout.
    fn upload_mips(
        &self,
        levels: &[render::tga::Image],
        sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        self.upload_mips_at(levels, sampler, &self.sprite_layout, 0)
    }

    /// [`Gpu::upload_mips`] under `layout`, the texture at `binding` and
    /// the sampler at the next.
    fn upload_mips_at(
        &self,
        levels: &[render::tga::Image],
        sampler: &wgpu::Sampler,
        layout: &wgpu::BindGroupLayout,
        binding: u32,
    ) -> wgpu::BindGroup {
        let base = &levels[0];
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("figure"),
            size: wgpu::Extent3d {
                width: base.width as u32,
                height: base.height as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: levels.len() as u32,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.texel_format(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        for (level, image) in levels.iter().enumerate() {
            let extent = wgpu::Extent3d {
                width: image.width as u32,
                height: image.height as u32,
                depth_or_array_layers: 1,
            };
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &tex,
                    mip_level: level as u32,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &image.rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(image.width as u32 * 4),
                    rows_per_image: Some(image.height as u32),
                },
                extent,
            );
        }
        let view = tex.create_view(&Default::default());
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("figure"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: binding + 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        })
    }

    /// Each patch mesh's corners as position, base texcoord and alpha
    /// texcoord (in the third attribute's `xy`), beside its base's and its
    /// alpha's bind groups (item 4139).
    fn patch_meshes(&self, meshes: &[crate::world::PatchMesh]) -> Vec<UploadedPatch> {
        self.patch_meshes_of(meshes.iter())
    }

    /// [`Self::patch_meshes`] over any run of meshes (item 4239: the
    /// surf's alone).
    fn patch_meshes_of<'m>(
        &self,
        meshes: impl Iterator<Item = &'m crate::world::PatchMesh>,
    ) -> Vec<UploadedPatch> {
        meshes
            .filter(|m| !m.positions.is_empty())
            .map(|m| {
                // Item 4278: a main layer under its maps carries each
                // corner's normal and tangent after the patch's attributes.
                let maps = m
                    .maps
                    .as_ref()
                    .filter(|mp| mp.frame.len() == m.positions.len());
                let stride = if maps.is_some() { MAPPED_STRIDE } else { 32 };
                let mut bytes = Vec::with_capacity(m.positions.len() * stride as usize);
                for (i, ((p, uv), a)) in
                    m.positions.iter().zip(&m.uvs).zip(&m.alpha_uvs).enumerate()
                {
                    // Item 4217: the corner's N·L in the third attribute's `z`.
                    let nl = m.light.get(i).copied().unwrap_or(crate::world::LIGHT_FLAT);
                    for f in p.iter().chain(uv).chain(a).chain(&[nl]) {
                        bytes.extend_from_slice(&f.to_le_bytes());
                    }
                    if let Some(mp) = maps {
                        for f in mp.frame[i].iter().flatten() {
                            bytes.extend_from_slice(&f.to_le_bytes());
                        }
                    }
                }
                let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("patch"),
                    size: bytes.len() as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                self.queue.write_buffer(&buf, 0, &bytes);
                let n = m.positions.len() as u32;
                (
                    buf,
                    n,
                    self.figure_texture(m.piece, &m.mips),
                    // Item 4275: the snowcap's stage 1 is linear and
                    // clamped, as a mountain's alpha is (unread: 4275b).
                    self.alpha_texture(
                        m.alpha,
                        &m.alpha_mips,
                        m.mountain
                            || m.piece == crate::world::SNOWCAP_PIECE
                            || crate::world::is_land_oil(m.piece),
                    ),
                    m.mountain,
                    patch_kind(m.piece, m.alpha),
                    maps.map(|mp| self.mtrl_texture(&mp.mtrl)),
                )
            })
            .collect()
    }

    /// `mountain_mtrl` as group 3's bind group (item 4278), uploaded on
    /// first use, under the figures' repeating sampler: stage 2 is sampled
    /// with stage 0's sampler (`SamplerA`) at stage 0's texcoord.
    fn mtrl_texture(&self, mips: &render::silhouette::Mips) -> wgpu::BindGroup {
        let mut cache = self.mtrl_textures.lock().unwrap();
        if let Some(b) = cache.as_ref() {
            return b.clone();
        }
        let b = self.upload_mips_at(&mips.levels, &self.figure_sampler, &self.mtrl_layout, 2);
        *cache = Some(b.clone());
        b
    }

    /// The particles' meshes on the GPU (item 4233), each beside its
    /// texture's bind group and whether its blend adds.
    fn particle_meshes(
        &self,
        meshes: &[crate::particle::ParticleMesh],
    ) -> Vec<(wgpu::Buffer, u32, wgpu::BindGroup, bool)> {
        meshes
            .iter()
            .filter(|m| !m.positions.is_empty())
            .map(|m| {
                let mut bytes = Vec::with_capacity(m.positions.len() * 36);
                for ((p, uv), c) in m.positions.iter().zip(&m.uvs).zip(&m.colours) {
                    for f in p.iter().chain(uv).chain(c) {
                        bytes.extend_from_slice(&f.to_le_bytes());
                    }
                }
                let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("particles"),
                    size: bytes.len() as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                self.queue.write_buffer(&buf, 0, &bytes);
                let t = self
                    .particle_textures
                    .lock()
                    .unwrap()
                    .entry(m.key)
                    .or_insert_with(|| {
                        self.upload_mips(std::slice::from_ref(&*m.texture), &self.tree_sampler)
                    })
                    .clone();
                (buf, m.positions.len() as u32, t, m.additive)
            })
            .collect()
    }

    /// Each textured figure's corners as position and texcoord, beside its
    /// texture's bind group.
    fn textured_meshes(&self, meshes: &[crate::world::TexturedMesh]) -> Vec<Uploaded> {
        meshes
            .iter()
            .filter(|m| !m.positions.is_empty())
            .map(|m| {
                let mut bytes = Vec::with_capacity(m.positions.len() * 32);
                let owner = render::silhouette::team_unit(m.who).map(|c| f32::from(c) / 255.0);
                for (i, (p, uv)) in m.positions.iter().zip(&m.uvs).enumerate() {
                    // Item 4217: a ground mesh's owner is the corner's N·L;
                    // item 4345: a lit figure's too.
                    let owner = if m.piece < 0 || !m.light.is_empty() {
                        let nl = m.light.get(i).copied().unwrap_or(crate::world::LIGHT_FLAT);
                        [nl, 0.0, 0.0]
                    } else {
                        owner
                    };
                    for f in p.iter().chain(uv).chain(&owner) {
                        bytes.extend_from_slice(&f.to_le_bytes());
                    }
                }
                let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("textured figure"),
                    size: bytes.len() as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                self.queue.write_buffer(&buf, 0, &bytes);
                let n = m.positions.len().min(m.uvs.len()) as u32;
                (buf, n, self.figure_texture(m.piece, &m.mips))
            })
            .collect()
    }

    /// The trees' meshes on the GPU (item 4173), each beside its
    /// texture's bind group under the clamped sampler, uploaded once.
    fn tree_meshes(&self, meshes: &[crate::world::TexturedMesh]) -> Vec<Uploaded> {
        self.flat_meshes(meshes, &self.tree_textures, [0.0; 3])
    }

    /// Each shadow mesh (item 4182) as [`Gpu::tree_meshes`] uploads a
    /// tree's, the vertex alpha 1 (`BLACK`'s 255) in the third attribute.
    fn decal_meshes(&self, meshes: &[crate::world::TexturedMesh]) -> Vec<Uploaded> {
        self.flat_meshes(meshes, &self.decal_textures, [1.0, 0.0, 0.0])
    }

    /// Meshes of corners and texcoords, `third` every corner's third
    /// attribute, each texture on its level 0 under the clamped sampler,
    /// uploaded once into `textures`.
    fn flat_meshes(
        &self,
        meshes: &[crate::world::TexturedMesh],
        textures: &std::sync::Mutex<std::collections::BTreeMap<i32, wgpu::BindGroup>>,
        third: [f32; 3],
    ) -> Vec<Uploaded> {
        meshes
            .iter()
            .filter(|m| !m.positions.is_empty())
            .map(|m| {
                let mut bytes = Vec::with_capacity(m.positions.len() * 32);
                for (p, uv) in m.positions.iter().zip(&m.uvs) {
                    for f in p.iter().chain(uv).chain(&third) {
                        bytes.extend_from_slice(&f.to_le_bytes());
                    }
                }
                let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("tree"),
                    size: bytes.len() as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                self.queue.write_buffer(&buf, 0, &bytes);
                let n = m.positions.len().min(m.uvs.len()) as u32;
                let mut cache = textures.lock().unwrap();
                let t = cache
                    .entry(m.piece)
                    .or_insert_with(|| self.upload_mips(&m.mips.levels[..1], &self.tree_sampler))
                    .clone();
                (buf, n, t)
            })
            .collect()
    }

    /// The sprites' quads, two triangles each in screen pixels at the
    /// client's depth for the anchor, beside each one's bind group.
    fn sprite_quads(
        &self,
        sprites: &[crate::world::SpriteDrawing],
    ) -> Vec<(wgpu::Buffer, wgpu::BindGroup)> {
        sprites
            .iter()
            .map(|sp| {
                let (w, h) = (sp.image.image.width as f32, sp.image.image.height as f32);
                let (l, t) = (sp.at.left as f32, sp.at.top as f32);
                let (r, b) = (l + w * sp.at.scale, t + h * sp.at.scale);
                let z =
                    ((sp.depth - DEPTH_RANGE.0) / (DEPTH_RANGE.1 - DEPTH_RANGE.0)).clamp(0.0, 1.0);
                let v = [
                    [l, t, 0.0, 0.0],
                    [r, t, 1.0, 0.0],
                    [l, b, 0.0, 1.0],
                    [r, t, 1.0, 0.0],
                    [r, b, 1.0, 1.0],
                    [l, b, 0.0, 1.0],
                ];
                let mut bytes = Vec::with_capacity(6 * 20);
                for [x, y, u, vv] in v {
                    for f in [x, y, z, u, vv] {
                        bytes.extend_from_slice(&f.to_le_bytes());
                    }
                }
                let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("sprite quad"),
                    size: bytes.len() as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                self.queue.write_buffer(&buf, 0, &bytes);
                (buf, self.texture((sp.piece, sp.who), &sp.image.image))
            })
            .collect()
    }

    fn vertices(&self, (pos, col): Tris<'_>) -> Option<wgpu::Buffer> {
        if pos.is_empty() {
            return None;
        }
        let mut bytes = Vec::with_capacity(pos.len() * 24);
        for (p, c) in pos.iter().zip(col) {
            for v in p.iter().chain(c) {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
        }
        let b = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertices"),
            size: bytes.len() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&b, 0, &bytes);
        Some(b)
    }

    /// Draw one frame into `target` at the camera.
    pub fn draw(
        &self,
        target: &wgpu::TextureView,
        size: (u32, u32),
        camera: &render::Camera,
        frame: &Frame<'_>,
    ) {
        let m = camera.to_screen_matrix().0;
        let r = camera.render_matrix().0;
        let mut u: Vec<f32> = m.to_vec();
        u.extend([r[2], r[6], r[10], r[14]]);
        u.extend([DEPTH_RANGE.0, DEPTH_RANGE.1, 0.0, 0.0]);
        let eye = camera.eye_position();
        u.extend([eye[0], eye[1], eye[2], 1.0]);
        let bytes: Vec<u8> = u.iter().flat_map(|v| v.to_le_bytes()).collect();
        self.queue.write_buffer(&self.uniform, 0, &bytes);
        let screen: Vec<u8> = [size.0 as f32, size.1 as f32, 0.0, 0.0]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        self.queue.write_buffer(&self.sprite_uniform, 0, &screen);
        let quads = self.sprite_quads(frame.sprites);
        let textured = self.textured_meshes(frame.textured);
        let trees = self.tree_meshes(frame.trees);
        let shadows = self.decal_meshes(frame.shadows);
        let particles = self.particle_meshes(frame.particles);
        let depth = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth.create_view(&Default::default());
        let exposed = self.expose.frame(&self.device, self.format, size);
        let scene_view = exposed.scene.create_view(&Default::default());
        let mut cache = self.terrain.lock().unwrap();
        if cache
            .as_ref()
            .is_none_or(|(v, _, _)| *v != frame.terrain_version)
        {
            *cache = self
                .vertices(frame.terrain)
                .map(|b| (frame.terrain_version, b, frame.terrain.0.len() as u32));
        }
        let mut ground = self.ground_meshes.lock().unwrap();
        if ground
            .as_ref()
            .is_none_or(|(v, _, _, _, _)| *v != frame.terrain_version)
        {
            *ground = Some((
                frame.terrain_version,
                self.textured_meshes(frame.ground),
                self.patch_meshes(frame.patch),
                frame.patch_version,
                frame.surf_version,
            ));
        } else if let Some(g) = ground.as_mut().filter(|g| g.3 != frame.patch_version) {
            g.2 = self.patch_meshes(frame.patch);
            g.3 = frame.patch_version;
            g.4 = frame.surf_version;
        } else if let Some(g) = ground.as_mut().filter(|g| g.4 != frame.surf_version) {
            // Item 4239: the surf alone moved. Its lists are uploaded again
            // and put back after the water, where the world lays them.
            let surf = frame
                .patch
                .iter()
                .filter(|m| crate::world::is_surf(m.piece));
            g.2.retain(|p| p.5 != PatchKind::Surf);
            let at =
                g.2.iter()
                    .position(|p| p.5 == PatchKind::Water)
                    .map_or(g.2.len(), |w| w + 1);
            let uploaded = self.patch_meshes_of(surf);
            g.2.splice(at..at, uploaded);
            g.4 = frame.surf_version;
        }
        let figures = self.vertices(frame.figures);
        let overlay = self.vertices(frame.overlay);
        let under_pos: Vec<[f32; 3]> = frame.underlay.iter().flat_map(|l| l.pos.clone()).collect();
        let under_col: Vec<[f32; 3]> = frame.underlay.iter().flat_map(|l| l.col.clone()).collect();
        let underlay = self.vertices((&under_pos, &under_col));
        let screen_pos: Vec<[f32; 3]> = frame.screen.iter().flat_map(|l| l.pos.clone()).collect();
        let screen_col: Vec<[f32; 3]> = frame.screen.iter().flat_map(|l| l.col.clone()).collect();
        let screen = self.vertices((&screen_pos, &screen_col));
        let mut enc = self.device.create_command_encoder(&Default::default());
        let empty = (Vec::new(), Vec::new());
        let (meshes, patches) = ground
            .as_ref()
            .filter(|(_, m, _, _, _)| !m.is_empty())
            .map_or((&empty.0, &empty.1), |(_, m, p, _, _)| (m, p));
        // Item 4182: with shadows, the ground up to its first mountain layer
        // is drawn off screen, the decals into the keep map, and the shade
        // pass composes the two onto the target before everything else —
        // the card's draws 116–119 between the land and the mountains (121).
        let split = patches.iter().position(|p| p.4).unwrap_or(patches.len());
        let shade = (!shadows.is_empty()).then(|| self.shade_targets(size));
        // Item 4344 (§3.30, "The fish under the water"): with the objects
        // before the water, the water and the surf's lists after it are cut
        // out of the ground and drawn after the frame pass, over a copy
        // that holds the objects; the patches after them (a mountain's
        // layers) go to the frame pass, before the objects, as
        // `render_mountains` comes before `render_water`.
        let empty_patches: &[UploadedPatch] = &[];
        let (ground_run, frame_runs, water_run): (
            &[UploadedPatch],
            [&[UploadedPatch]; 2],
            &[UploadedPatch],
        ) = match patches
            .iter()
            .position(|p| p.5 == PatchKind::Water)
            .filter(|_| frame.objects_before_water)
        {
            Some(w) => {
                let e = w
                    + 1
                    + patches[w + 1..]
                        .iter()
                        .take_while(|p| p.5 == PatchKind::Surf)
                        .count();
                let cut = if shade.is_some() { split.min(w) } else { w };
                (
                    &patches[..cut],
                    [&patches[cut..w], &patches[e..]],
                    &patches[w..e],
                )
            }
            None if shade.is_some() => (
                &patches[..split],
                [&patches[split..], empty_patches],
                empty_patches,
            ),
            None => (&patches[..], [empty_patches, empty_patches], empty_patches),
        };
        if let Some((under, keep, _)) = &shade {
            {
                let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("keep"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &keep.create_view(&Default::default()),
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                pass.set_bind_group(0, &self.bind, &[]);
                pass.set_pipeline(&self.decal);
                for (b, n, t) in &shadows {
                    pass.set_bind_group(1, t, &[]);
                    pass.set_vertex_buffer(0, b.slice(..));
                    pass.draw(0..*n, 0..1);
                }
            }
            self.ground_passes(
                &mut enc,
                under,
                &depth_view,
                size,
                cache.as_ref(),
                meshes,
                ground_run,
                frame.land_hdr,
                frame.surf_hdr,
                frame.border_hdr,
            );
        } else {
            // Item 4232: the ground in passes of its own, so the water can
            // read the frame under it; the frame pass loads it.
            self.ground_passes(
                &mut enc,
                &exposed.scene,
                &depth_view,
                size,
                cache.as_ref(),
                meshes,
                ground_run,
                frame.land_hdr,
                frame.surf_hdr,
                frame.border_hdr,
            );
        }
        {
            let colour_load = if shade.is_some() {
                wgpu::LoadOp::Clear(CLEAR)
            } else {
                wgpu::LoadOp::Load
            };
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("frame"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &scene_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: colour_load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if let Some((_, _, bind)) = &shade {
                pass.set_pipeline(&self.shade);
                pass.set_bind_group(0, bind, &[]);
                pass.draw(0..3, 0..1);
            }
            for run in frame_runs.into_iter().filter(|r| !r.is_empty()) {
                self.draw_ground(&mut pass, None, None, run, None);
            }
            pass.set_bind_group(0, &self.bind, &[]);
            // Item 4401: the underlays after the ground, before the
            // buildings, as `BuildOut::draw_underlays` comes before the
            // objects draw over it.
            // Item 4421: a layer a row of the band's texture, its alpha
            // the texel's times the vertex's (`MODULATE(TEXTURE, CURRENT)`).
            if let Some(b) = &underlay {
                pass.set_pipeline(&self.underlay);
                pass.set_vertex_buffer(0, b.slice(..));
                let mut at = 0u32;
                for l in frame.underlay {
                    let n = l.pos.len() as u32;
                    let a = f64::from(l.alpha) / 255.0 * f64::from(UNDERLAY_ALPHA) / 255.0;
                    pass.set_blend_constant(wgpu::Color {
                        r: a,
                        g: a,
                        b: a,
                        a,
                    });
                    pass.draw(at..at + n, 0..1);
                    at += n;
                }
            }
            pass.set_pipeline(&self.world);
            if !quads.is_empty() {
                pass.set_pipeline(&self.sprite);
                pass.set_bind_group(0, &self.sprite_bind, &[]);
                for (b, t) in &quads {
                    pass.set_bind_group(1, t, &[]);
                    pass.set_vertex_buffer(0, b.slice(..));
                    pass.draw(0..6, 0..1);
                }
                pass.set_bind_group(0, &self.bind, &[]);
                pass.set_pipeline(&self.world);
            }
            if let Some(b) = &figures {
                pass.set_vertex_buffer(0, b.slice(..));
                pass.draw(0..frame.figures.0.len() as u32, 0..1);
            }
            // `flush_trees` (§3.7, "The order"), textured (item 4173), before
            // the guys pass that tests their depth.
            if !trees.is_empty() {
                pass.set_pipeline(&self.tree);
                for (b, n, t) in &trees {
                    pass.set_bind_group(1, t, &[]);
                    pass.set_vertex_buffer(0, b.slice(..));
                    pass.draw(0..*n, 0..1);
                }
            }
            if !textured.is_empty() {
                // The guys pass after the trees (item 4085): each figure
                // where it is behind, in its owner's colour (4103), and
                // lit; the silhouette's place is `silhouette_order`'s.
                let passes = match self.silhouette_order {
                    SilhouetteOrder::Before => [&self.silhouette, &self.textured],
                    SilhouetteOrder::After => [&self.textured, &self.silhouette],
                };
                // Item 4345: a lit figure's own pass in the textured's place.
                let lit: Vec<bool> = frame
                    .textured
                    .iter()
                    .filter(|m| !m.positions.is_empty())
                    .map(|m| !m.light.is_empty())
                    .collect();
                for pipeline in passes {
                    for (k, (b, n, t)) in textured.iter().enumerate() {
                        let own = std::ptr::eq(pipeline, &self.textured) && lit[k];
                        pass.set_pipeline(if own { &self.textured_lit } else { pipeline });
                        pass.set_bind_group(1, t, &[]);
                        pass.set_vertex_buffer(0, b.slice(..));
                        pass.draw(0..*n, 0..1);
                    }
                }
            }
        }
        // Item 4344: the water and the surf over the objects.
        if !water_run.is_empty() {
            self.water_passes(
                &mut enc,
                &exposed.scene,
                &depth_view,
                size,
                water_run,
                (frame.surf_hdr, frame.border_hdr),
            );
        }
        // Item 4233 (§3.25): the particles after the figures, as draws
        // 216–229 come after them on the card; item 4237: blended in HDR.
        // Item 4241: the light target is cleared even when nothing burns,
        // since the expose pass reads it.
        self.plume_passes(&mut enc, &exposed, &depth_view, size, &particles);
        // Item 4212 (§3.21): the tone map's white at the frame's mean, then
        // the overlay, which the tone map does not take.
        self.expose.measure(&mut enc, &exposed);
        self.expose.bloom(&mut enc, &exposed);
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shown"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(CLEAR),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Discard,
                    }),
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.expose.show(&mut pass, &exposed);
            // Item 4410: the plates over the tone-mapped scene, as
            // `BuildOut::draw_overlays` lays them through the interface's
            // renderer, under the debug overlay and the HUD.
            if let Some(b) = &screen {
                pass.set_bind_group(0, &self.bind, &[]);
                pass.set_pipeline(&self.screen);
                pass.set_vertex_buffer(0, b.slice(..));
                let mut at = 0u32;
                for l in frame.screen {
                    let n = l.pos.len() as u32;
                    let a = f64::from(l.alpha) / 255.0;
                    pass.set_blend_constant(wgpu::Color {
                        r: a,
                        g: a,
                        b: a,
                        a,
                    });
                    pass.draw(at..at + n, 0..1);
                    at += n;
                }
            }
            if let Some(b) = &overlay {
                pass.set_bind_group(0, &self.bind, &[]);
                pass.set_pipeline(&self.overlay);
                pass.set_vertex_buffer(0, b.slice(..));
                pass.draw(0..frame.overlay.0.len() as u32, 0..1);
            }
        }
        self.queue.submit([enc.finish()]);
    }

    /// **The particles in HDR** (item 4237; `docs/RENDERER.md` §3.25):
    /// each drawn into the plume target under the frame's depth, then the
    /// frame copied and composed with it through the tone map onto
    /// `target`.
    fn plume_passes(
        &self,
        enc: &mut wgpu::CommandEncoder,
        exposed: &crate::expose::ExposeFrame,
        depth_view: &wgpu::TextureView,
        size: (u32, u32),
        particles: &[(wgpu::Buffer, u32, wgpu::BindGroup, bool)],
    ) {
        let extent = wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        };
        let (target, light, under) = (&exposed.scene, &exposed.light, &exposed.under);
        let light_view = light.create_view(&Default::default());
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("plume"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &light_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_bind_group(0, &self.bind, &[]);
            for (b, n, t, additive) in particles {
                pass.set_pipeline(if *additive { &self.fire } else { &self.smoke });
                pass.set_bind_group(1, t, &[]);
                pass.set_vertex_buffer(0, b.slice(..));
                pass.draw(0..*n, 0..1);
            }
        }
        if particles.is_empty() {
            return;
        }
        enc.copy_texture_to_texture(target.as_image_copy(), under.as_image_copy(), extent);
        let bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("plume"),
            layout: &self.shade_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &under.create_view(&Default::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&light_view),
                },
            ],
        });
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("plume composed"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.create_view(&Default::default()),
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.plume);
        pass.set_bind_group(0, &bind, &[]);
        pass.draw(0..3, 0..1);
    }

    /// The heightfield and the base layer (`meshes`, when given), then the
    /// patches given — the transition patch after the base (item 4139,
    /// §3.9) and a mountain's layers after it (item 4141, §3.10) — leaving
    /// the world's pipeline and camera bound.
    /// **The ground's passes** (item 4232): the base layer and the patches
    /// up to the water into `target`, cleared; then, when there is water,
    /// the frame so far copied to the lake texture, and the water and the
    /// patches after it in a second pass over the first, the water reading
    /// the copy (`fs_water`).
    #[allow(clippy::too_many_arguments)]
    fn ground_passes(
        &self,
        enc: &mut wgpu::CommandEncoder,
        target: &wgpu::Texture,
        depth_view: &wgpu::TextureView,
        size: (u32, u32),
        terrain: Option<&(u32, wgpu::Buffer, u32)>,
        meshes: &[Uploaded],
        patches: &[UploadedPatch],
        land_hdr: bool,
        surf_hdr: bool,
        border_hdr: bool,
    ) {
        let view = target.create_view(&Default::default());
        let water = patches.iter().position(|p| p.5 == PatchKind::Water);
        let (first, rest) = patches.split_at(water.unwrap_or(patches.len()));
        // Item 4327: the land pass's faded base, floor, transition patches
        // and coral, one run in the world's order, blended on the HDR target.
        let in_hdr = |p: &UploadedPatch| {
            !p.4 && matches!(p.5, PatchKind::Faded | PatchKind::Floor | PatchKind::Lit)
        };
        let (first, run, after) = match first.iter().position(in_hdr).filter(|_| land_hdr) {
            Some(s) => {
                let e = s + first[s..].iter().take_while(|p| in_hdr(p)).count();
                (&first[..s], &first[s..e], &first[e..])
            }
            None => (first, &first[..0], &first[..0]),
        };
        let extent = wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        };
        // Item 4379: the borders' runs leave the frame pass for the HDR
        // target, so the first pass stops at the first of them.
        let k = first
            .iter()
            .position(|p| border_hdr && p.5 == PatchKind::Border)
            .unwrap_or(first.len());
        {
            let mut pass = begin(enc, &view, depth_view, true);
            self.draw_ground(&mut pass, terrain, Some(meshes), &first[..k], None);
        }
        let draws = (target, &view, depth_view, extent, border_hdr);
        self.draw_split(enc, draws, &first[k..], None);
        if !run.is_empty() {
            self.land_hdr_pass(enc, target, &view, depth_view, extent, run);
            self.draw_split(enc, draws, after, None);
        }
        self.water_passes(enc, target, depth_view, size, rest, (surf_hdr, border_hdr));
    }

    /// **Patches over the frame, the borders' runs in HDR** (item 4379;
    /// `docs/RENDERER.md` §3.29, "The band's colour"): each run of
    /// [`PatchKind::Border`] drawn over a copy of the frame by
    /// `border_over`, as the card's draws 135 and 136 blend on its HDR
    /// target before the tone map, and the patches between over the frame
    /// as [`Self::draw_ground`] draws them. With `border_hdr` off, all of them over the frame.
    fn draw_split(
        &self,
        enc: &mut wgpu::CommandEncoder,
        (target, view, depth_view, extent, border_hdr): (
            &wgpu::Texture,
            &wgpu::TextureView,
            &wgpu::TextureView,
            wgpu::Extent3d,
            bool,
        ),
        patches: &[UploadedPatch],
        lake: Option<&wgpu::BindGroup>,
    ) {
        let is_border = |p: &UploadedPatch| border_hdr && p.5 == PatchKind::Border;
        let mut i = 0;
        while i < patches.len() {
            let j = patches[i..]
                .iter()
                .position(is_border)
                .map_or(patches.len(), |k| i + k);
            if j > i {
                let mut pass = begin(enc, view, depth_view, false);
                self.draw_ground(&mut pass, None, None, &patches[i..j], lake);
            }
            let e = j + patches[j..].iter().take_while(|p| is_border(p)).count();
            if e > j {
                let lake = self.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("border_under"),
                    size: extent,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: self.format,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                enc.copy_texture_to_texture(target.as_image_copy(), lake.as_image_copy(), extent);
                let under = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("border_under"),
                    layout: &self.lake_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &lake.create_view(&Default::default()),
                        ),
                    }],
                });
                let mut pass = begin(enc, view, depth_view, false);
                pass.set_bind_group(0, &self.bind, &[]);
                pass.set_bind_group(3, &under, &[]);
                pass.set_pipeline(&self.border_over);
                for (b, n, t, a, _, _, _) in &patches[j..e] {
                    pass.set_bind_group(1, t, &[]);
                    pass.set_bind_group(2, a, &[]);
                    pass.set_vertex_buffer(0, b.slice(..));
                    pass.draw(0..*n, 0..1);
                }
            }
            i = e;
        }
    }

    /// **The water and what follows it** (items 4232, 4239, 4338): the
    /// frame so far copied to the lake texture, the water reading the copy
    /// (`fs_water`), and each run of the surf's lists on the HDR target (or
    /// each list over a copy of its own), the other patches over the
    /// water's copy. Item 4344: after the frame pass, with the objects
    /// before the water.
    fn water_passes(
        &self,
        enc: &mut wgpu::CommandEncoder,
        target: &wgpu::Texture,
        depth_view: &wgpu::TextureView,
        size: (u32, u32),
        rest: &[UploadedPatch],
        (surf_hdr, border_hdr): (bool, bool),
    ) {
        if rest.is_empty() {
            return;
        }
        let view = target.create_view(&Default::default());
        let extent = wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        };
        let copy = |enc: &mut wgpu::CommandEncoder| {
            let lake = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("lake"),
                size: extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            enc.copy_texture_to_texture(target.as_image_copy(), lake.as_image_copy(), extent);
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("lake"),
                layout: &self.lake_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &lake.create_view(&Default::default()),
                    ),
                }],
            })
        };
        let bind = copy(enc);
        // Item 4239: each of the surf's lists over a copy of the frame as
        // the lists before it leave it; the rest over the water's copy.
        let mut i = 0;
        while i < rest.len() {
            let j = rest[i..]
                .iter()
                .position(|p| p.5 == PatchKind::Surf)
                .map_or(rest.len(), |k| i + k);
            if j > i {
                let draws = (target, &view, depth_view, extent, border_hdr);
                self.draw_split(enc, draws, &rest[i..j], Some(&bind));
            }
            if j < rest.len() && surf_hdr {
                // Item 4338: the run of lists from here, on the HDR target.
                let e = j + rest[j..]
                    .iter()
                    .take_while(|p| p.5 == PatchKind::Surf)
                    .count();
                self.land_hdr_pass(enc, target, &view, depth_view, extent, &rest[j..e]);
                i = e;
            } else if j < rest.len() {
                let under = copy(enc);
                let mut pass = begin(enc, &view, depth_view, false);
                self.draw_ground(&mut pass, None, None, &rest[j..j + 1], Some(&under));
                i = j + 1;
            } else {
                i = j;
            }
        }
    }

    /// **The land pass in HDR** (item 4327; `docs/RENDERER.md` §3.27, "The
    /// patches over the faded land in HDR"): the frame so far taken back
    /// through the curve onto an HDR target (`fs_untone`), `run` drawn there
    /// with the target's own source-alpha blend on the lit values, and the
    /// target taken through the curve back onto the frame (`fs_retone`):
    /// `T(T⁻¹(D) · (1 − a) + L · a)` per layer, as the card's draws 0–131.
    fn land_hdr_pass(
        &self,
        enc: &mut wgpu::CommandEncoder,
        target: &wgpu::Texture,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        extent: wgpu::Extent3d,
        run: &[UploadedPatch],
    ) {
        let texture = |label: &str, format, usage| {
            self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
        };
        let bind = |t: &wgpu::Texture| {
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("land_hdr"),
                layout: &self.lake_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &t.create_view(&Default::default()),
                    ),
                }],
            })
        };
        let under = texture(
            "land_under",
            self.format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        enc.copy_texture_to_texture(target.as_image_copy(), under.as_image_copy(), extent);
        let hdr = texture(
            "land_hdr",
            LAND_HDR_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let hdr_view = hdr.create_view(&Default::default());
        let full = |enc: &mut wgpu::CommandEncoder,
                    to: &wgpu::TextureView,
                    pipeline: &wgpu::RenderPipeline,
                    from: &wgpu::BindGroup| {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("land_hdr_full"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: to,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(3, from, &[]);
            pass.draw(0..3, 0..1);
        };
        full(enc, &hdr_view, &self.untone, &bind(&under));
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("land_hdr"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &hdr_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_bind_group(0, &self.bind, &[]);
            for (b, n, t, a, _, kind, _) in run {
                pass.set_pipeline(match kind {
                    PatchKind::Faded => &self.hdr_faded,
                    PatchKind::Floor => &self.hdr_floor,
                    PatchKind::Surf => &self.hdr_surf,
                    _ => &self.hdr_lit,
                });
                pass.set_bind_group(1, t, &[]);
                pass.set_bind_group(2, a, &[]);
                pass.set_vertex_buffer(0, b.slice(..));
                pass.draw(0..*n, 0..1);
            }
        }
        full(enc, view, &self.retone, &bind(&hdr));
    }

    fn draw_ground(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        terrain: Option<&(u32, wgpu::Buffer, u32)>,
        meshes: Option<&[Uploaded]>,
        patches: &[UploadedPatch],
        lake: Option<&wgpu::BindGroup>,
    ) {
        pass.set_bind_group(0, &self.bind, &[]);
        pass.set_pipeline(&self.world);
        if let Some((_, b, n)) = terrain {
            pass.set_vertex_buffer(0, b.slice(..));
            pass.draw(0..*n, 0..1);
        }
        if let Some(meshes) = meshes.filter(|m| !m.is_empty()) {
            pass.set_pipeline(&self.ground);
            for (b, n, t) in meshes {
                pass.set_bind_group(1, t, &[]);
                pass.set_vertex_buffer(0, b.slice(..));
                pass.draw(0..*n, 0..1);
            }
        }
        for (b, n, t, a, mountain, kind, mtrl) in patches {
            let pipeline = match (*mountain, kind) {
                (true, _) if mtrl.is_some() => {
                    pass.set_bind_group(3, mtrl.as_ref(), &[]);
                    &self.mountain_mapped
                }
                (true, _) => &self.mountain,
                (_, PatchKind::Plain) => &self.patch,
                (_, PatchKind::Lit) => &self.patch_lit,
                (_, PatchKind::Faded) => &self.faded,
                (_, PatchKind::Floor) => &self.floor,
                (_, PatchKind::Snowcap) => &self.snowcap,
                (_, PatchKind::Billboard) => &self.billboard,
                (_, PatchKind::Edge) => &self.edge,
                (_, PatchKind::Border) => &self.border,
                (_, PatchKind::Surf) => {
                    // Over a copy of the frame under it: drawn only by
                    // `ground_passes`' surf steps.
                    let Some(lake) = lake else { continue };
                    pass.set_bind_group(3, lake, &[]);
                    &self.surf
                }
                (_, PatchKind::Water) => {
                    // The water reads the frame under it: drawn only by
                    // `ground_passes`' second pass.
                    let Some(lake) = lake else { continue };
                    pass.set_bind_group(3, lake, &[]);
                    &self.water
                }
            };
            pass.set_pipeline(pipeline);
            pass.set_bind_group(1, t, &[]);
            pass.set_bind_group(2, a, &[]);
            pass.set_vertex_buffer(0, b.slice(..));
            pass.draw(0..*n, 0..1);
        }
        pass.set_pipeline(&self.world);
    }

    /// The shade pass's targets at `size` (item 4182): the ground under the
    /// decals in the frame's format, the keep map, and their bind group.
    fn shade_targets(&self, size: (u32, u32)) -> (wgpu::Texture, wgpu::Texture, wgpu::BindGroup) {
        let texture = |label: &str, format: wgpu::TextureFormat| {
            self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            })
        };
        let under = texture("under", self.format);
        // Item 4232: the water's pass copies it (`ground_passes`).
        let keep = texture("keep", KEEP_FORMAT);
        let (uv, kv) = (
            under.create_view(&Default::default()),
            keep.create_view(&Default::default()),
        );
        let bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shade"),
            layout: &self.shade_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&uv),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&kv),
                },
            ],
        });
        (under, keep, bind)
    }

    /// An off-screen frame to draw into (`client --headless`, item 4161).
    pub fn target(&self, size: (u32, u32)) -> wgpu::Texture {
        self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("off-screen"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
    }

    /// Wait until the GPU has finished what was submitted: an off-screen
    /// frame's end, where a window's is its present.
    pub fn wait(&self) {
        let _ = self.device.poll(wgpu::PollType::wait_indefinitely());
    }

    /// Draw one frame to a texture and read it back as rows of RGBA bytes.
    pub fn still(&self, size: (u32, u32), camera: &render::Camera, frame: &Frame<'_>) -> Vec<u8> {
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("still"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        self.draw(&tex.create_view(&Default::default()), size, camera, frame);
        let row = (size.0 * 4).div_ceil(256) * 256;
        let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: u64::from(row * size.1),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = self.device.create_command_encoder(&Default::default());
        enc.copy_texture_to_buffer(
            tex.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buf,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(row),
                    rows_per_image: Some(size.1),
                },
            },
            wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([enc.finish()]);
        let slice = buf.slice(..);
        let done = Arc::new(std::sync::Mutex::new(None));
        let d = done.clone();
        slice.map_async(wgpu::MapMode::Read, move |r| *d.lock().unwrap() = Some(r));
        let _ = self.device.poll(wgpu::PollType::wait_indefinitely());
        let mut out = Vec::with_capacity((size.0 * size.1 * 4) as usize);
        if matches!(*done.lock().unwrap(), Some(Ok(())))
            && let Ok(data) = slice.get_mapped_range()
        {
            for y in 0..size.1 {
                let s = (y * row) as usize;
                out.extend_from_slice(&data[s..s + (size.0 * 4) as usize]);
            }
        }
        out
    }
}

/// A 24-bit BMP of RGBA rows, bottom-up as the format keeps them — the
/// still's file, read by any viewer, with no image dependency.
pub fn bmp(rgba: &[u8], size: (u32, u32)) -> Vec<u8> {
    let (w, h) = size;
    let row = (w * 3).div_ceil(4) * 4;
    let data = row * h;
    let mut b = Vec::with_capacity((54 + data) as usize);
    b.extend_from_slice(b"BM");
    b.extend_from_slice(&(54 + data).to_le_bytes());
    b.extend_from_slice(&[0; 4]);
    b.extend_from_slice(&54u32.to_le_bytes());
    b.extend_from_slice(&40u32.to_le_bytes());
    b.extend_from_slice(&(w as i32).to_le_bytes());
    b.extend_from_slice(&(h as i32).to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&24u16.to_le_bytes());
    b.extend_from_slice(&0u32.to_le_bytes());
    b.extend_from_slice(&data.to_le_bytes());
    b.extend_from_slice(&[0; 16]);
    for y in (0..h).rev() {
        let start = b.len();
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            b.extend_from_slice(&[rgba[i + 2], rgba[i + 1], rgba[i]]);
        }
        b.resize(start + row as usize, 0);
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Item 4103: the silhouette's pass draws at the alpha the painter's
    /// `render::silhouette::silhouette_blend` takes.
    #[test]
    fn the_silhouette_pass_draws_at_the_painter_s_alpha() {
        let alpha = format!(
            "vec4<f32>(i.owner, {}.0 / 255.0)",
            render::silhouette::SILHOUETTE_ALPHA
        );
        assert!(TEXTURED_SHADER.contains(&alpha), "{alpha}");
    }

    /// Item 4127: the ground draws the texel's colour, its alpha laid over
    /// nothing (`docs/RENDERER.md` §3.6). Item 4212 (§3.21): through the
    /// land's lit branch.
    #[test]
    fn the_ground_draws_the_texel_lit() {
        assert!(TEXTURED_SHADER.contains(
            "fn fs_ground(i: VOut) -> @location(0) vec4<f32> {\n    \
             return vec4<f32>(land_lit(textureSample(tex, samp, i.uv).rgb, i.owner.x), 1.0);"
        ));
    }

    /// Item 4232: the water template's every hole is filled, once.
    #[test]
    fn the_water_template_s_holes_are_filled() {
        for h in WATER_HOLES {
            assert_eq!(WATER_SHADER.matches(&format!("{{{h}}}")).count(), 1, "{h}");
        }
        let t = water_shader();
        assert!(
            !t.chars()
                .zip(t.chars().skip(1))
                .any(|(a, b)| a == '{' && b.is_ascii_alphabetic()),
            "a hole the template names and the code does not fill"
        );
    }

    /// Item 4212: the land template's every hole is filled, once.
    #[test]
    fn the_land_template_s_holes_are_filled() {
        for h in LAND_HOLES {
            assert_eq!(LAND_SHADER.matches(&format!("{{{h}}}")).count(), 1, "{h}");
        }
        let t = land_shader();
        assert!(
            !t.chars()
                .zip(t.chars().skip(1))
                .any(|(a, b)| a == '{' && b.is_ascii_alphabetic()),
            "a hole the template names and the code does not fill"
        );
    }

    /// Item 4172: the mountain's ADDSIGNED runs on the stored bytes, not
    /// on the sRGB view's linear samples (the still's body read a third of
    /// the photograph's brightness with it in linear space). Item 4171: the
    /// textures and targets are plain `Unorm`, so the sample is the byte and
    /// the sum is taken as it stands, with no encode or decode around it.
    /// Item 4269: lit by the range's own N·L with the vertex white
    /// (`land_lit_white`).
    #[test]
    fn the_mountain_adds_on_the_stored_bytes() {
        assert!(PATCH_SHADER.contains(
            "return vec4<f32>(land_lit_white(clamp(t0.rgb + t1.rgb - vec3<f32>(0.5), vec3<f32>(0.0), vec3<f32>(1.0)), i.nl), a);"
        ));
        assert!(!PATCH_SHADER.contains("encoded("));
    }

    /// Item 4173: the tree's pass runs the painter's arithmetic on the
    /// bytes — the stage on the sample, the power, the scale and the white
    /// the painter's constants give — with no encode or decode around it.
    #[test]
    fn the_tree_pass_is_the_painter_s() {
        let t = tree_shader();
        assert!(t.contains("let stage = clamp(t.rgb + vec3<f32>(0.5019608 - 0.5)"));
        assert!(t.contains("pow(stage, vec3<f32>(1.9)) * 5.0"));
        // Item 4221: blended by the texel's alpha (§3.23).
        assert!(t.contains("return vec4<f32>(toned, t.a);"));
        assert!(!t.contains("encoded("));
        assert!(t.contains("if (t.a <= 0.0)"));
    }

    /// Parse and validate one shader as the device would take it.
    fn validated(name: &str, src: &str) -> Result<(), String> {
        use wgpu::naga::{front::wgsl, valid};
        let module =
            wgsl::parse_str(src).map_err(|e| format!("{name}: {}", e.emit_to_string(src)))?;
        valid::Validator::new(
            valid::ValidationFlags::all(),
            valid::Capabilities::default(),
        )
        .validate(&module)
        .map_err(|e| format!("{name}: {e:?}"))?;
        Ok(())
    }

    /// Item 4181: every shader module `Gpu::new` builds parses and
    /// validates under `naga` (the `wgpu::naga` the device compiles with),
    /// each source as [`shader_sources`] assembles it — the one list
    /// `Gpu::new` compiles.
    #[test]
    fn every_shader_parses_and_validates() {
        for (name, src) in shader_sources() {
            validated(name, &src).unwrap_or_else(|e| panic!("{e}"));
        }
        // The check can fail: one character changed is refused.
        let broken = SHADER.replacen(
            "return vec4<f32>(i.col, 1.0);",
            "return vec4<f32>(i.col, 1.0)",
            1,
        );
        assert_ne!(broken, SHADER);
        assert!(validated("world.wgsl, perturbed", &broken).is_err());
    }

    /// Item 4181: the tree template's every hole is filled, once, and no
    /// other brace pair in it names a constant.
    #[test]
    fn the_tree_template_s_holes_are_filled() {
        for h in TREE_HOLES {
            assert_eq!(TREE_SHADER.matches(&format!("{{{h}}}")).count(), 1, "{h}");
        }
        let t = tree_shader();
        for h in TREE_HOLES {
            assert!(!t.contains(&format!("{{{h}}}")), "{h} left unfilled");
        }
        assert!(
            !t.chars()
                .zip(t.chars().skip(1))
                .any(|(a, b)| a == '{' && b.is_ascii_alphabetic()),
            "a hole the template names and the code does not fill"
        );
    }

    /// The shade template's every hole is filled, once, and no other
    /// brace pair in it names a constant.
    #[test]
    fn the_shade_template_s_holes_are_filled() {
        for h in SHADE_HOLES {
            assert_eq!(SHADE_SHADER.matches(&format!("{{{h}}}")).count(), 1, "{h}");
        }
        let t = shade_shader();
        for h in SHADE_HOLES {
            assert!(!t.contains(&format!("{{{h}}}")), "{h} left unfilled");
        }
        assert!(
            !t.chars()
                .zip(t.chars().skip(1))
                .any(|(a, b)| a == '{' && b.is_ascii_alphabetic()),
            "a hole the template names and the code does not fill"
        );
    }

    /// Item 4237: the particle template's every hole is filled, once, and
    /// no other brace pair in it names a constant.
    #[test]
    fn the_particle_template_s_holes_are_filled() {
        for h in PARTICLE_HOLES {
            assert_eq!(
                PARTICLE_SHADER.matches(&format!("{{{h}}}")).count(),
                1,
                "{h}"
            );
        }
        let t = particle_shader();
        for h in PARTICLE_HOLES {
            assert!(!t.contains(&format!("{{{h}}}")), "{h} left unfilled");
        }
        assert!(
            !t.chars()
                .zip(t.chars().skip(1))
                .any(|(a, b)| a == '{' && b.is_ascii_alphabetic()),
            "a hole the template names and the code does not fill"
        );
    }

    /// The text of `file` outside its holes, in order: a template's
    /// `{NAME}` (a brace, a letter, word characters, a brace) cut out.
    fn outside_holes(file: &str) -> Vec<&str> {
        let b = file.as_bytes();
        let (mut out, mut from, mut i) = (Vec::new(), 0, 0);
        while i < b.len() {
            if b[i] == b'{' && b.get(i + 1).is_some_and(u8::is_ascii_alphabetic) {
                let end = b[i + 1..]
                    .iter()
                    .position(|c| !(c.is_ascii_alphanumeric() || *c == b'_'))
                    .map(|n| i + 1 + n);
                if let Some(end) = end.filter(|&e| b[e] == b'}') {
                    out.push(&file[from..i]);
                    from = end + 1;
                    i = from;
                    continue;
                }
            }
            i += 1;
        }
        out.push(&file[from..]);
        out
    }

    /// Whether `file` is part of `src`: every piece of it outside its
    /// holes found in `src`, in order.
    fn is_part_of(file: &str, src: &str) -> bool {
        let mut at = 0;
        outside_holes(file)
            .iter()
            .all(|piece| match src[at..].find(piece) {
                Some(n) => {
                    at += n + piece.len();
                    true
                }
                None => false,
            })
    }

    /// Every `*.wgsl` under `crates/client/shaders/` is part of a source
    /// in [`shader_sources`] — a template's text outside its holes, in
    /// order: a file the list forgets is caught here.
    #[test]
    fn every_shader_file_is_in_the_list() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("shaders");
        let sources = shader_sources();
        let mut files = 0;
        let mut missing = Vec::new();
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|e| e.to_str()) != Some("wgsl") {
                continue;
            }
            files += 1;
            let text = std::fs::read_to_string(&path).unwrap();
            if !sources.iter().any(|(_, src)| is_part_of(&text, src)) {
                missing.push(path.file_name().unwrap().to_string_lossy().into_owned());
            }
        }
        missing.sort();
        assert!(files > 0, "no shader under {}", dir.display());
        assert!(
            missing.is_empty(),
            "not in any validated source: {missing:?}"
        );
    }

    /// The crate builds shader modules at one call site, the map over
    /// [`shader_sources`] in `Gpu::new`, which destructures the list into
    /// its seven modules: a module built by hand elsewhere, outside the
    /// validated list, is caught here. (Before the list, seven call sites
    /// against a hand list of five.)
    #[test]
    fn every_module_built_is_validated() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let call = concat!("create_shader_", "module(");
        let mut built = 0;
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                built += std::fs::read_to_string(&path)
                    .unwrap()
                    .matches(call)
                    .count();
            }
        }
        assert_eq!(built, 1, "call sites building a shader module");
        assert_eq!(shader_sources().len(), 11);
    }

    #[test]
    fn a_bmp_is_bottom_up_bgr_with_padded_rows() {
        // Two pixels wide, two high: red, green over blue, white.
        let rgba = [
            255, 0, 0, 255, 0, 255, 0, 255, //
            0, 0, 255, 255, 255, 255, 255, 255,
        ];
        let b = bmp(&rgba, (2, 2));
        assert_eq!(&b[..2], b"BM");
        assert_eq!(b.len(), 54 + 8 * 2, "6 bytes a row padded to 8");
        assert_eq!(
            &b[54..60],
            &[255, 0, 0, 255, 255, 255],
            "the bottom row first"
        );
        assert_eq!(&b[62..68], &[0, 0, 255, 0, 255, 0]);
    }
}
