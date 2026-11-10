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
    /// The buildings drawn as their sprites (item 4076).
    pub sprites: &'a [crate::world::SpriteDrawing],
}

pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    world: wgpu::RenderPipeline,
    overlay: wgpu::RenderPipeline,
    uniform: wgpu::Buffer,
    bind: wgpu::BindGroup,
    format: wgpu::TextureFormat,
    terrain: std::sync::Mutex<Option<(u32, wgpu::Buffer, u32)>>,
    sprite: wgpu::RenderPipeline,
    sprite_uniform: wgpu::Buffer,
    sprite_bind: wgpu::BindGroup,
    sprite_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    /// Each building piece's sprite on the GPU, uploaded once.
    textures: std::sync::Mutex<std::collections::BTreeMap<i32, wgpu::BindGroup>>,
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
        Ok(Gpu {
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
            if let Some(b) = &overlay {
                pass.set_pipeline(&self.overlay);
                pass.set_vertex_buffer(0, b.slice(..));
                pass.draw(0..frame.overlay.0.len() as u32, 0..1);
            }
        }
        self.queue.submit([enc.finish()]);
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
