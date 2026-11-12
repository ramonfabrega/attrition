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

const SHADER: &str = r#"
struct U {
    m: mat4x4<f32>,
    d: vec4<f32>,
    range: vec4<f32>,
};
@group(0) @binding(0) var<uniform> u: U;

struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) col: vec3<f32>,
};

@vertex
fn vs_world(@location(0) p: vec3<f32>, @location(1) c: vec3<f32>) -> VOut {
    let v = vec4<f32>(p, 1.0);
    let clip = u.m * v;
    let depth = dot(v, u.d);
    let z = clamp((depth - u.range.x) / (u.range.y - u.range.x), 0.0, 1.0);
    var o: VOut;
    o.pos = vec4<f32>(clip.xy, z * clip.w, clip.w);
    o.col = c;
    return o;
}

@vertex
fn vs_overlay(@location(0) p: vec3<f32>, @location(1) c: vec3<f32>) -> VOut {
    var o: VOut;
    o.pos = vec4<f32>(p.xy, 0.0, 1.0);
    o.col = c;
    return o;
}

@fragment
fn fs(i: VOut) -> @location(0) vec4<f32> {
    return vec4<f32>(i.col, 1.0);
}
"#;

/// **The sprites' shader** (item 4076): a building's `texture=` as a quad in
/// screen pixels at `render::draw::SpriteAt`, at the draw list's depth for
/// its anchor, a pixel kept where its alpha is half or more. It is blitted
/// over the ground (depth compare always) and writes the anchor's depth, so
/// a figure nearer than the anchor draws over it and one further does not — the pixels
/// `render::silhouette::Stamp` counts. No team colour (`build_sprite_color`)
/// is applied.
const SPRITE_SHADER: &str = r#"
struct S {
    screen: vec4<f32>,
};
@group(0) @binding(0) var<uniform> s: S;
@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var samp: sampler;

struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_sprite(@location(0) p: vec3<f32>, @location(1) uv: vec2<f32>) -> VOut {
    var o: VOut;
    o.pos = vec4<f32>(p.x / s.screen.x * 2.0 - 1.0, 1.0 - p.y / s.screen.y * 2.0, p.z, 1.0);
    o.uv = uv;
    return o;
}

@fragment
fn fs_sprite(i: VOut) -> @location(0) vec4<f32> {
    let c = textureSample(tex, samp, i.uv);
    if c.a < 0.5 {
        discard;
    }
    return vec4<f32>(c.rgb, 1.0);
}
"#;

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
const TEXTURED_SHADER: &str = r#"
struct U {
    m: mat4x4<f32>,
    d: vec4<f32>,
    range: vec4<f32>,
};
@group(0) @binding(0) var<uniform> u: U;
@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var samp: sampler;

struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) owner: vec3<f32>,
};

@vertex
fn vs_textured(
    @location(0) p: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) owner: vec3<f32>,
) -> VOut {
    let v = vec4<f32>(p, 1.0);
    let clip = u.m * v;
    let depth = dot(v, u.d);
    let z = clamp((depth - u.range.x) / (u.range.y - u.range.x), 0.0, 1.0);
    var o: VOut;
    o.pos = vec4<f32>(clip.xy, z * clip.w, clip.w);
    o.uv = uv;
    o.owner = owner;
    return o;
}

@fragment
fn fs_textured(i: VOut) -> @location(0) vec4<f32> {
    let t = textureSample(tex, samp, i.uv);
    return vec4<f32>(mix(i.owner, t.rgb, t.a), 1.0);
}

@fragment
fn fs_ground(i: VOut) -> @location(0) vec4<f32> {
    return vec4<f32>(textureSample(tex, samp, i.uv).rgb, 1.0);
}

@fragment
fn fs_silhouette(i: VOut) -> @location(0) vec4<f32> {
    return vec4<f32>(i.owner, 128.0 / 255.0);
}
"#;

/// **The transition patch's pass** (item 4139; `docs/RENDERER.md` §3.9),
/// `TerrainOut::render`'s `showtrans` block as its `RenderState` reads:
/// stage 0 the patch's base land texture, under the vertex white
/// (MODULATE with DIFFUSE, the texel alone); stage 1 the alpha, its A
/// taken (MODULATE with TEXTURE and CURRENT, the diffuse alpha 1), read on
/// level 0, `MIP_POINT` and `TEXADD_CLAMP` — so a nearest, clamped sampler
/// on the alpha's own level 0; blended `SRCALPHA`, `INVSRCALPHA` over the
/// base already drawn. The vertex is the textured layout's: the alpha's
/// texcoord rides in the third attribute's `xy`.
const PATCH_SHADER: &str = r#"
struct U {
    m: mat4x4<f32>,
    d: vec4<f32>,
    range: vec4<f32>,
};
@group(0) @binding(0) var<uniform> u: U;
@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var samp: sampler;
@group(2) @binding(0) var alpha: texture_2d<f32>;
@group(2) @binding(1) var alpha_samp: sampler;

struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) auv: vec2<f32>,
};

@vertex
fn vs_patch(
    @location(0) p: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) auv: vec3<f32>,
) -> VOut {
    let v = vec4<f32>(p, 1.0);
    let clip = u.m * v;
    let depth = dot(v, u.d);
    let z = clamp((depth - u.range.x) / (u.range.y - u.range.x), 0.0, 1.0);
    var o: VOut;
    o.pos = vec4<f32>(clip.xy, z * clip.w, clip.w);
    o.uv = uv;
    o.auv = auv.xy;
    return o;
}

@fragment
fn fs_patch(i: VOut) -> @location(0) vec4<f32> {
    let a = textureSampleLevel(alpha, alpha_samp, i.auv, 0.0).a;
    return vec4<f32>(textureSample(tex, samp, i.uv).rgb, a);
}

// Item 4141 (`docs/RENDERER.md` §3.10): one layer of a mountain's own draw,
// `MountainsOut::mountain_blend` — stage 0 the tileset's texture (MODULATE
// with the vertex white, A its own); stage 1 the range's alpha, ADDSIGNED
// on the colour and MODULATE on A; the alpha test GREATER 0.
@fragment
fn fs_mountain(i: VOut) -> @location(0) vec4<f32> {
    let t0 = textureSample(tex, samp, i.uv);
    let t1 = textureSampleLevel(alpha, alpha_samp, i.auv, 0.0);
    let a = t0.a * t1.a;
    if (a <= 0.0) {
        discard;
    }
    return vec4<f32>(clamp(t0.rgb + t1.rgb - vec3<f32>(0.5), vec3<f32>(0.0), vec3<f32>(1.0)), a);
}
"#;

/// The camera-space depth range mapped onto [0, 1]: wider than any map.
const DEPTH_RANGE: (f32, f32) = (-200_000.0, 200_000.0);

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

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
    /// The ground's base layer (item 4127), kept on the GPU with the
    /// terrain at `terrain_version`.
    pub ground: &'a [crate::world::TexturedMesh],
    /// The transition patch over the base (item 4139), kept with it.
    pub patch: &'a [crate::world::PatchMesh],
    /// The buildings drawn as their sprites (item 4076).
    pub sprites: &'a [crate::world::SpriteDrawing],
}

/// A mesh on the GPU: its vertices, their count and its texture.
type Uploaded = (wgpu::Buffer, u32, wgpu::BindGroup);

/// A patch mesh on the GPU: [`Uploaded`], its alpha's bind group and
/// whether it is a mountain's layer (item 4141).
type UploadedPatch = (wgpu::Buffer, u32, wgpu::BindGroup, wgpu::BindGroup, bool);

/// The ground's base layer and its patch on the GPU, at a terrain version.
type GroundUploads = (u32, Vec<Uploaded>, Vec<UploadedPatch>);

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
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("client"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
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
        let sprite_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sprites"),
            source: wgpu::ShaderSource::Wgsl(SPRITE_SHADER.into()),
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
        let textured_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("textured"),
            source: wgpu::ShaderSource::Wgsl(TEXTURED_SHADER.into()),
        });
        let figure_pass = |label: &str,
                           entry: &str,
                           write: bool,
                           compare: wgpu::CompareFunction,
                           blend: Option<wgpu::BlendState>| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&sprite_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &textured_module,
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
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &textured_module,
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
        let textured = figure_pass(
            "textured",
            "fs_textured",
            true,
            wgpu::CompareFunction::Less,
            None,
        );
        let silhouette = figure_pass(
            "silhouette",
            "fs_silhouette",
            false,
            wgpu::CompareFunction::Greater,
            Some(wgpu::BlendState::ALPHA_BLENDING),
        );
        let ground = figure_pass(
            "ground",
            "fs_ground",
            true,
            wgpu::CompareFunction::Less,
            None,
        );
        let patch_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("patch"),
            source: wgpu::ShaderSource::Wgsl(PATCH_SHADER.into()),
        });
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
        Ok(Gpu {
            ground,
            ground_meshes: std::sync::Mutex::new(None),
            patch,
            alpha_sampler,
            mountain,
            mountain_sampler,
            alpha_textures: std::sync::Mutex::new(Default::default()),
            textured,
            silhouette,
            figure_sampler,
            figure_textures: std::sync::Mutex::new(Default::default()),
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
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
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
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
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
            .is_none_or(|(v, _, _)| *v != frame.terrain_version)
        {
            *ground = Some((
                frame.terrain_version,
                self.textured_meshes(frame.ground),
                self.patch_meshes(frame.patch),
            ));
        }
        let figures = self.vertices(frame.figures);
        let overlay = self.vertices(frame.overlay);
        let mut enc = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("frame"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.01,
                            g: 0.01,
                            b: 0.015,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_bind_group(0, &self.bind, &[]);
            pass.set_pipeline(&self.world);
            if let Some((_, b, n)) = cache.as_ref() {
                pass.set_vertex_buffer(0, b.slice(..));
                pass.draw(0..*n, 0..1);
            }
            if let Some((_, meshes, patches)) = ground.as_ref().filter(|(_, m, _)| !m.is_empty()) {
                pass.set_pipeline(&self.ground);
                for (b, n, t) in meshes {
                    pass.set_bind_group(1, t, &[]);
                    pass.set_vertex_buffer(0, b.slice(..));
                    pass.draw(0..*n, 0..1);
                }
                // The patch after the base (item 4139, §3.9).
                if !patches.is_empty() {
                    // Item 4141 (§3.10): a mountain's layers after it.
                    for (b, n, t, a, mountain) in patches {
                        pass.set_pipeline(if *mountain {
                            &self.mountain
                        } else {
                            &self.patch
                        });
                        pass.set_bind_group(1, t, &[]);
                        pass.set_bind_group(2, a, &[]);
                        pass.set_vertex_buffer(0, b.slice(..));
                        pass.draw(0..*n, 0..1);
                    }
                }
                pass.set_pipeline(&self.world);
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
            if !textured.is_empty() {
                pass.set_pipeline(&self.textured);
                for (b, n, t) in &textured {
                    pass.set_bind_group(1, t, &[]);
                    pass.set_vertex_buffer(0, b.slice(..));
                    pass.draw(0..*n, 0..1);
                }
                // The guys pass after the trees (item 4085): each figure
                // again where it is behind, in its owner's colour (4103).
                pass.set_pipeline(&self.silhouette);
                for (b, n, t) in &textured {
                    pass.set_bind_group(1, t, &[]);
                    pass.set_vertex_buffer(0, b.slice(..));
                    pass.draw(0..*n, 0..1);
                }
            }
            if let Some(b) = &overlay {
                pass.set_pipeline(&self.overlay);
                pass.set_vertex_buffer(0, b.slice(..));
                pass.draw(0..frame.overlay.0.len() as u32, 0..1);
            }
        }
        self.queue.submit([enc.finish()]);
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

    /// Item 4127: the ground draws the texel's colour alone, its alpha
    /// laid over nothing (`docs/RENDERER.md` §3.6).
    #[test]
    fn the_ground_draws_the_texel_s_colour_alone() {
        assert!(TEXTURED_SHADER.contains(
            "fn fs_ground(i: VOut) -> @location(0) vec4<f32> {\n    \
             return vec4<f32>(textureSample(tex, samp, i.uv).rgb, 1.0);"
        ));
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
