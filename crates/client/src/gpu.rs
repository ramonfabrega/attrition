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
const LAND_HOLES: [&str; 9] = [
    "LIT_POWER",
    "HDR_SCALE",
    "LAND_DIFFUSE",
    "N_DOT_L",
    "AMBIENT_R",
    "AMBIENT_G",
    "AMBIENT_B",
    "EXPOSURE",
    "white",
];

/// [`LAND_SHADER`] with its holes filled.
fn land_shader() -> String {
    use render::silhouette::{
        EXPOSURE, HDR_SCALE, LAND_DIFFUSE, LAND_N_DOT_L, LIT_POWER, MIN_LUMINANCE, SCENE_AMBIENT,
        WHITE_POINT, filmic,
    };
    let white = 1.0 / filmic(MIN_LUMINANCE * WHITE_POINT);
    LAND_SHADER
        .replace("{LIT_POWER}", &format!("{LIT_POWER:?}"))
        .replace("{HDR_SCALE}", &format!("{HDR_SCALE:?}"))
        .replace("{LAND_DIFFUSE}", &format!("{LAND_DIFFUSE:?}"))
        .replace("{N_DOT_L}", &format!("{LAND_N_DOT_L:?}"))
        .replace("{AMBIENT_R}", &format!("{:?}", SCENE_AMBIENT[0]))
        .replace("{AMBIENT_G}", &format!("{:?}", SCENE_AMBIENT[1]))
        .replace("{AMBIENT_B}", &format!("{:?}", SCENE_AMBIENT[2]))
        .replace("{EXPOSURE}", &format!("{EXPOSURE:?}"))
        .replace("{white}", &format!("{white:?}"))
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
fn shader_sources() -> [(&'static str, String); 9] {
    [
        ("client", SHADER.to_string()),
        ("sprites", SPRITE_SHADER.to_string()),
        ("textured", format!("{TEXTURED_SHADER}{}", land_shader())),
        (
            "decal",
            format!("{TEXTURED_SHADER}{}{DECAL_SHADER}", land_shader()),
        ),
        (
            "tree",
            format!("{TEXTURED_SHADER}{}{}", land_shader(), tree_shader()),
        ),
        ("shade", shade_shader()),
        ("patch", format!("{PATCH_SHADER}{}", land_shader())),
        ("luminance", crate::expose::luminance_shader()),
        ("expose", crate::expose::expose_shader()),
    ]
}

/// The camera-space depth range mapped onto [0, 1]: wider than any map.
const DEPTH_RANGE: (f32, f32) = (-200_000.0, 200_000.0);

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

/// One frame's triangles: the terrain's (kept on the GPU while its version
/// stands), the figures' and the overlay's.
pub struct Frame<'a> {
    pub terrain_version: u32,
    pub terrain: Tris<'a>,
    pub figures: Tris<'a>,
    pub overlay: Tris<'a>,
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
    /// The buildings drawn as their sprites (item 4076).
    pub sprites: &'a [crate::world::SpriteDrawing],
    /// The shadows (item 4182), one mesh a decal texture.
    pub shadows: &'a [crate::world::TexturedMesh],
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

/// A mesh on the GPU: its vertices, their count and its texture.
type Uploaded = (wgpu::Buffer, u32, wgpu::BindGroup);

/// A patch mesh on the GPU: [`Uploaded`], its alpha's bind group and
/// whether it is a mountain's layer (item 4141).
type UploadedPatch = (
    wgpu::Buffer,
    u32,
    wgpu::BindGroup,
    wgpu::BindGroup,
    bool,
    bool,
);

/// **Whether a patch mesh takes the land's light** (item 4212;
/// `docs/RENDERER.md` §3.21): a land texture's layer (keyed `-1 − index`,
/// the faded land and the transition patches) and the coral, which the
/// card draws `ENABLE_LIGHTING`; not the water, the floor's shade, the
/// roads or the weathering, which the client draws as their colours stand.
fn lit_patch(piece: i32) -> bool {
    use crate::world::{CORAL_PIECE, WATER_PIECE};
    (WATER_PIECE + 1..0).contains(&piece) || piece == CORAL_PIECE
}

/// The ground's base layer and its patch on the GPU, at a terrain version.
/// Item 4178: and the patches' own version, which the screen's ramp moves.
type GroundUploads = (u32, Vec<Uploaded>, Vec<UploadedPatch>, u32);

pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    world: wgpu::RenderPipeline,
    overlay: wgpu::RenderPipeline,
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
    /// The land's patches, lit (item 4212).
    patch_lit: wgpu::RenderPipeline,
    alpha_sampler: wgpu::Sampler,
    /// A mountain's layers (item 4141, §3.10): their pass, and the alphas'
    /// linear, clamped sampler (`MIP_LINEAR`, `TEXADD_CLAMP`).
    mountain: wgpu::RenderPipeline,
    mountain_sampler: wgpu::Sampler,
    alpha_textures: std::sync::Mutex<std::collections::BTreeMap<usize, wgpu::BindGroup>>,
    sprite: wgpu::RenderPipeline,
    sprite_uniform: wgpu::Buffer,
    sprite_bind: wgpu::BindGroup,
    sprite_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    /// Each building piece's sprite on the GPU, uploaded once.
    textures: std::sync::Mutex<std::collections::BTreeMap<i32, wgpu::BindGroup>>,
    /// The figures' textured pipeline and its sampler (item 4099).
    textured: wgpu::RenderPipeline,
    /// The figures' silhouette pass (item 4103): `render_states[1]`'s
    /// depth Greater, no depth write, source-alpha blend.
    silhouette: wgpu::RenderPipeline,
    figure_sampler: wgpu::Sampler,
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
        ] = shader_sources().map(|(label, source)| {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            })
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera"),
            size: 96,
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
        let pipeline = |entry: &str, depth: bool| {
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
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
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
                    blend: None,
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
            None,
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
        let blended = |label: &str, entry: &str| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&patch_layout),
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
                    depth_write_enabled: Some(false),
                    depth_compare: Some(wgpu::CompareFunction::LessEqual),
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
        let patch = blended("patch", "fs_patch");
        let patch_lit = blended("patch_lit", "fs_patch_lit");
        let mountain = blended("mountain", "fs_mountain");
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
            alpha_sampler,
            mountain,
            mountain_sampler,
            alpha_textures: std::sync::Mutex::new(Default::default()),
            textured,
            silhouette,
            figure_sampler,
            figure_textures: std::sync::Mutex::new(Default::default()),
            silhouette_order: SilhouetteOrder::Before,
            sprite,
            sprite_uniform,
            sprite_bind,
            sprite_layout,
            sampler,
            textures: std::sync::Mutex::new(Default::default()),
            world: pipeline("vs_world", true),
            overlay: pipeline("vs_overlay", false),
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
    fn texture(&self, piece: i32, image: &render::tga::Image) -> wgpu::BindGroup {
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
        let b = self.upload_mips(&mips.levels, &self.figure_sampler);
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
            layout: &self.sprite_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        })
    }

    /// Each patch mesh's corners as position, base texcoord and alpha
    /// texcoord (in the third attribute's `xy`), beside its base's and its
    /// alpha's bind groups (item 4139).
    fn patch_meshes(&self, meshes: &[crate::world::PatchMesh]) -> Vec<UploadedPatch> {
        meshes
            .iter()
            .filter(|m| !m.positions.is_empty())
            .map(|m| {
                let mut bytes = Vec::with_capacity(m.positions.len() * 32);
                for ((p, uv), a) in m.positions.iter().zip(&m.uvs).zip(&m.alpha_uvs) {
                    for f in p.iter().chain(uv).chain(a).chain(&[0.0]) {
                        bytes.extend_from_slice(&f.to_le_bytes());
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
                    self.alpha_texture(m.alpha, &m.alpha_mips, m.mountain),
                    m.mountain,
                    lit_patch(m.piece),
                )
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
                for (p, uv) in m.positions.iter().zip(&m.uvs) {
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
                (buf, self.texture(sp.piece, &sp.image.image))
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
            .is_none_or(|(v, _, _, _)| *v != frame.terrain_version)
        {
            *ground = Some((
                frame.terrain_version,
                self.textured_meshes(frame.ground),
                self.patch_meshes(frame.patch),
                frame.patch_version,
            ));
        } else if let Some(g) = ground.as_mut().filter(|g| g.3 != frame.patch_version) {
            g.2 = self.patch_meshes(frame.patch);
            g.3 = frame.patch_version;
        }
        let figures = self.vertices(frame.figures);
        let overlay = self.vertices(frame.overlay);
        let mut enc = self.device.create_command_encoder(&Default::default());
        let empty = (Vec::new(), Vec::new());
        let (meshes, patches) = ground
            .as_ref()
            .filter(|(_, m, _, _)| !m.is_empty())
            .map_or((&empty.0, &empty.1), |(_, m, p, _)| (m, p));
        // Item 4182: with shadows, the ground up to its first mountain layer
        // is drawn off screen, the decals into the keep map, and the shade
        // pass composes the two onto the target before everything else —
        // the card's draws 116–119 between the land and the mountains (121).
        let split = patches.iter().position(|p| p.4).unwrap_or(patches.len());
        let shade = (!shadows.is_empty()).then(|| self.shade_targets(size));
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
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("under"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &under.create_view(&Default::default()),
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
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: wgpu::StoreOp::Store,
                    }),
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.draw_ground(&mut pass, cache.as_ref(), Some(meshes), &patches[..split]);
        }
        {
            let (depth_load, stencil_load) = if shade.is_some() {
                (wgpu::LoadOp::Load, wgpu::LoadOp::Load)
            } else {
                (wgpu::LoadOp::Clear(1.0), wgpu::LoadOp::Clear(0))
            };
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("frame"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &scene_view,
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
                        load: depth_load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: stencil_load,
                        store: wgpu::StoreOp::Store,
                    }),
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            match &shade {
                Some((_, _, bind)) => {
                    pass.set_pipeline(&self.shade);
                    pass.set_bind_group(0, bind, &[]);
                    pass.draw(0..3, 0..1);
                    self.draw_ground(&mut pass, None, None, &patches[split..]);
                }
                None => self.draw_ground(&mut pass, cache.as_ref(), Some(meshes), patches),
            }
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
                for pipeline in passes {
                    pass.set_pipeline(pipeline);
                    for (b, n, t) in &textured {
                        pass.set_bind_group(1, t, &[]);
                        pass.set_vertex_buffer(0, b.slice(..));
                        pass.draw(0..*n, 0..1);
                    }
                }
            }
        }
        // Item 4212 (§3.21): the tone map's white at the frame's mean, then
        // the overlay, which the tone map does not take.
        self.expose.measure(&mut enc, &exposed);
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
            if let Some(b) = &overlay {
                pass.set_bind_group(0, &self.bind, &[]);
                pass.set_pipeline(&self.overlay);
                pass.set_vertex_buffer(0, b.slice(..));
                pass.draw(0..frame.overlay.0.len() as u32, 0..1);
            }
        }
        self.queue.submit([enc.finish()]);
    }

    /// The heightfield and the base layer (`meshes`, when given), then the
    /// patches given — the transition patch after the base (item 4139,
    /// §3.9) and a mountain's layers after it (item 4141, §3.10) — leaving
    /// the world's pipeline and camera bound.
    fn draw_ground(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        terrain: Option<&(u32, wgpu::Buffer, u32)>,
        meshes: Option<&[Uploaded]>,
        patches: &[UploadedPatch],
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
        for (b, n, t, a, mountain, lit) in patches {
            pass.set_pipeline(if *mountain {
                &self.mountain
            } else if *lit {
                &self.patch_lit
            } else {
                &self.patch
            });
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
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
        };
        let under = texture("under", self.format);
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
             return vec4<f32>(land_lit(textureSample(tex, samp, i.uv).rgb), 1.0);"
        ));
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
    #[test]
    fn the_mountain_adds_on_the_stored_bytes() {
        assert!(PATCH_SHADER.contains(
            "return vec4<f32>(land_lit(clamp(t0.rgb + t1.rgb - vec3<f32>(0.5), vec3<f32>(0.0), vec3<f32>(1.0))), a);"
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
        assert!(t.contains("return vec4<f32>(toned, 1.0);"));
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
        assert_eq!(shader_sources().len(), 9);
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
