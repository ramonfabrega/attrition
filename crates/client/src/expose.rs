//! **The tone map's white, taken from the frame** (item 4212;
//! `docs/RENDERER.md` §3.21). `ps_tonemap.hlsl` scales the whole HDR frame
//! by `1 / tonemap(max(avgLuminance, gMinLuminance) · gWhitePoint)`, the
//! mean the luminance target's mip 10 holds (`ps_luminance.hlsl`). The
//! client's draws are each the tone map's output at the floor
//! (`MIN_LUMINANCE`), so the frame is drawn off screen, its mean HDR
//! luminance taken through the curve's inverse on the GPU
//! (`shaders/luminance.wgsl`), and the frame shown scaled by the white's
//! ratio (`shaders/expose.wgsl`) before the overlay.
//!
//! Item 4241 (§3.25): `ps_tonemap.hlsl` scales the curve's output by the
//! white and the target clamps after it, so a pixel over the floor's white
//! reaches 255 where the client's frame, clamped at the floor, stopped at
//! the white's ratio (229 at run1559b's mean). The particles' light and the
//! frame under it ([`ExposeFrame::light`], [`ExposeFrame::under`]) are kept
//! to the expose pass, which composes a plume's pixel again without the
//! floor's clamp before it scales.
//!
//! Item 4241: the bloom. Both packets hold `section_scales[DRAW_SECTION_BLOOM]`
//! at `FULL_SECTION`, so `Scene::render` runs the tone mapper with it:
//! `ps_tonemap.hlsl`'s bright pass into a full-size target, `do_blur`'s
//! two passes of `ps_gaussian.hlsl` at half size (`shaders/bloom.wgsl`),
//! and `ps_blend_screen.hlsl` over the toned frame (`fs_expose`).

/// The luminance pass's template.
pub(crate) const LUMINANCE_SHADER: &str = include_str!("../shaders/luminance.wgsl");
/// The expose pass's template.
pub(crate) const EXPOSE_SHADER: &str = include_str!("../shaders/expose.wgsl");
/// The bloom's blur (item 4241).
pub(crate) const BLOOM_SHADER: &str = include_str!("../shaders/bloom.wgsl");

/// The names [`LUMINANCE_SHADER`] leaves a hole for, each filled once.
#[cfg(test)]
pub(crate) const LUMINANCE_HOLES: [&str; 5] = ["EXPOSURE", "white", "LUMA_R", "LUMA_G", "LUMA_B"];
/// The names [`EXPOSE_SHADER`] leaves a hole for.
#[cfg(test)]
pub(crate) const EXPOSE_HOLES: [&str; 10] = [
    "MIN_LUMINANCE",
    "WHITE_POINT",
    "EXPOSURE",
    "white",
    "LUMA_R",
    "LUMA_G",
    "LUMA_B",
    "BLOOM_STRENGTH",
    "BLOOM_TIGHTNESS",
    "BLOOM_THRESHOLD",
];

/// [`LUMINANCE_SHADER`] with its holes filled.
pub(crate) fn luminance_shader() -> String {
    use render::silhouette::{EXPOSURE, LUMA, MIN_LUMINANCE, WHITE_POINT, filmic};
    let white = 1.0 / filmic(MIN_LUMINANCE * WHITE_POINT);
    LUMINANCE_SHADER
        .replace("{EXPOSURE}", &format!("{EXPOSURE:?}"))
        .replace("{white}", &format!("{white:?}"))
        .replace("{LUMA_R}", &format!("{:?}", LUMA[0]))
        .replace("{LUMA_G}", &format!("{:?}", LUMA[1]))
        .replace("{LUMA_B}", &format!("{:?}", LUMA[2]))
}

/// [`EXPOSE_SHADER`] with its holes filled.
pub(crate) fn expose_shader() -> String {
    use render::silhouette::{
        BLOOM_STRENGTH, BLOOM_THRESHOLD, BLOOM_TIGHTNESS, EXPOSURE, LUMA, MIN_LUMINANCE,
        WHITE_POINT, filmic,
    };
    let white = 1.0 / filmic(MIN_LUMINANCE * WHITE_POINT);
    EXPOSE_SHADER
        .replace("{MIN_LUMINANCE}", &format!("{MIN_LUMINANCE:?}"))
        .replace("{WHITE_POINT}", &format!("{WHITE_POINT:?}"))
        .replace("{EXPOSURE}", &format!("{EXPOSURE:?}"))
        .replace("{white}", &format!("{white:?}"))
        .replace("{LUMA_R}", &format!("{:?}", LUMA[0]))
        .replace("{LUMA_G}", &format!("{:?}", LUMA[1]))
        .replace("{LUMA_B}", &format!("{:?}", LUMA[2]))
        .replace("{BLOOM_STRENGTH}", &format!("{BLOOM_STRENGTH:?}"))
        .replace("{BLOOM_TIGHTNESS}", &format!("{BLOOM_TIGHTNESS:?}"))
        .replace("{BLOOM_THRESHOLD}", &format!("{BLOOM_THRESHOLD:?}"))
}

/// **The bloom's targets' format** (item 4241): the original's are
/// `R10G10B10A2_UNORM`; the bright pass clamps as that target does, and
/// its ten bits are not modelled.
const BLOOM_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// **The particles' light target's format** (item 4237): HDR light, and
/// in its alpha what is left of the frame under it.
pub(crate) const PLUME_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// The passes and their layouts, built once.
pub(crate) struct Expose {
    luminance: wgpu::ComputePipeline,
    mean: wgpu::ComputePipeline,
    expose: wgpu::RenderPipeline,
    bright: wgpu::RenderPipeline,
    blur: wgpu::RenderPipeline,
    compute_layout: wgpu::BindGroupLayout,
    expose_layout: wgpu::BindGroupLayout,
    blur_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

/// One frame's: the off-screen scene the frame is drawn into, and the
/// bind group the expose pass reads it and its mean through.
pub(crate) struct ExposeFrame {
    pub scene: wgpu::Texture,
    /// The particles' light ([`PLUME_FORMAT`]), cleared to no light and
    /// the whole frame left when nothing burns (item 4241).
    pub light: wgpu::Texture,
    /// The scene as it stood before the particles were composed into it.
    pub under: wgpu::Texture,
    /// The bloom's bright pass, full size, and its two blurs, half size.
    bright: wgpu::Texture,
    across: wgpu::Texture,
    down: wgpu::Texture,
    blur_across: wgpu::BindGroup,
    blur_down: wgpu::BindGroup,
    partial: wgpu::Buffer,
    mean: wgpu::Buffer,
    groups: (u32, u32),
    compute: wgpu::BindGroup,
    pub show: wgpu::BindGroup,
}

const GROUP: u32 = 16;

impl Expose {
    pub fn new(
        device: &wgpu::Device,
        luminance_module: &wgpu::ShaderModule,
        expose_module: &wgpu::ShaderModule,
        bloom_module: &wgpu::ShaderModule,
        format: wgpu::TextureFormat,
        depth: wgpu::TextureFormat,
    ) -> Expose {
        let texture = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let filtered = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let sampled = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };
        let storage = |binding, read_only, visibility| wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let compute_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("luminance"),
            entries: &[
                texture(0),
                storage(1, false, wgpu::ShaderStages::COMPUTE),
                storage(2, false, wgpu::ShaderStages::COMPUTE),
            ],
        });
        let expose_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("expose"),
            entries: &[
                texture(0),
                storage(1, true, wgpu::ShaderStages::FRAGMENT),
                texture(2),
                texture(3),
                filtered(4),
                sampled(5),
            ],
        });
        let blur_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bloom"),
            entries: &[
                filtered(0),
                sampled(1),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let layout = |l: &wgpu::BindGroupLayout, label| {
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &[Some(l)],
                immediate_size: 0,
            })
        };
        let compute_pipeline_layout = layout(&compute_layout, "luminance");
        let compute = |entry: &str| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry),
                layout: Some(&compute_pipeline_layout),
                module: luminance_module,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: None,
            })
        };
        let expose_pipeline_layout = layout(&expose_layout, "expose");
        let expose = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("expose"),
            layout: Some(&expose_pipeline_layout),
            vertex: wgpu::VertexState {
                module: expose_module,
                entry_point: Some("vs_full"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: expose_module,
                entry_point: Some("fs_expose"),
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
        // Item 4241: the bright pass and the blur, each a full-screen
        // triangle into a bloom target, with no depth.
        let blur_pipeline_layout = layout(&blur_layout, "bloom");
        let full = |label, module, pipeline_layout, entry| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(pipeline_layout),
                vertex: wgpu::VertexState {
                    module,
                    entry_point: Some("vs_full"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module,
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: BLOOM_FORMAT,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let bright = full(
            "bright",
            expose_module,
            &expose_pipeline_layout,
            "fs_bright",
        );
        let blur = full("blur", bloom_module, &blur_pipeline_layout, "fs_blur");
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("bloom"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Expose {
            luminance: compute("cs_luminance"),
            mean: compute("cs_mean"),
            expose,
            bright,
            blur,
            compute_layout,
            expose_layout,
            blur_layout,
            sampler,
        }
    }

    /// A frame's scene texture, buffers and bind groups at `size`.
    pub fn frame(
        &self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        size: (u32, u32),
    ) -> ExposeFrame {
        let scene = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("scene"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            // Item 4232: the water's pass copies the scene under it.
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let side = |label, format, usage| {
            device.create_texture(&wgpu::TextureDescriptor {
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
                usage,
                view_formats: &[],
            })
        };
        let light = side(
            "plume",
            PLUME_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let under = side(
            "under the plume",
            format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        let target = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        let bright = side("bright", BLOOM_FORMAT, target);
        // `create_ping_pong_buffers`: half the screen's size, rounded down.
        let half = ((size.0 / 2).max(1), (size.1 / 2).max(1));
        let half_side = |label| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: half.0,
                    height: half.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: BLOOM_FORMAT,
                usage: target,
                view_formats: &[],
            })
        };
        let across = half_side("bloom across");
        let down = half_side("bloom down");
        let blur_group = |label, src: &wgpu::Texture, direction: [f32; 2]| {
            let values = [
                size.0 as f32,
                size.1 as f32,
                half.0 as f32,
                half.1 as f32,
                direction[0],
                direction[1],
                0.0,
                0.0,
            ];
            let uniform = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: 32,
                usage: wgpu::BufferUsages::UNIFORM,
                mapped_at_creation: true,
            });
            {
                let bytes: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
                uniform
                    .slice(..)
                    .get_mapped_range_mut()
                    .expect("the bloom's uniform maps at creation")
                    .copy_from_slice(&bytes);
            }
            uniform.unmap();
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &self.blur_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &src.create_view(&Default::default()),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: uniform.as_entire_binding(),
                    },
                ],
            })
        };
        // `do_blur`: across from the bright target, `gDirection` (1, 0);
        // down from that, (0, 1).
        let blur_across = blur_group("bloom across", &bright, [1.0, 0.0]);
        let blur_down = blur_group("bloom down", &across, [0.0, 1.0]);
        let groups = (size.0.div_ceil(GROUP), size.1.div_ceil(GROUP));
        let buffer = |label, n: u32| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: u64::from(n.max(1)) * 4,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            })
        };
        let partial = buffer("partial", groups.0 * groups.1);
        let mean = buffer("mean", 1);
        let view = scene.create_view(&Default::default());
        let compute = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("luminance"),
            layout: &self.compute_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: partial.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: mean.as_entire_binding(),
                },
            ],
        });
        let show = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("expose"),
            layout: &self.expose_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: mean.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        &light.create_view(&Default::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(
                        &under.create_view(&Default::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(
                        &down.create_view(&Default::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        ExposeFrame {
            scene,
            light,
            under,
            bright,
            across,
            down,
            blur_across,
            blur_down,
            partial,
            mean,
            groups,
            compute,
            show,
        }
    }

    /// The frame's mean, into its buffer: after the scene is drawn.
    pub fn measure(&self, enc: &mut wgpu::CommandEncoder, f: &ExposeFrame) {
        let _ = (&f.partial, &f.mean);
        let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("luminance"),
            timestamp_writes: None,
        });
        pass.set_bind_group(0, &f.compute, &[]);
        pass.set_pipeline(&self.luminance);
        pass.dispatch_workgroups(f.groups.0, f.groups.1, 1);
        pass.set_pipeline(&self.mean);
        pass.dispatch_workgroups(1, 1, 1);
    }

    /// **The bloom** (item 4241): the bright pass, then `do_blur`'s two
    /// passes; after [`Expose::measure`], since the bright pass reads the
    /// white.
    pub fn bloom(&self, enc: &mut wgpu::CommandEncoder, f: &ExposeFrame) {
        let passes: [(&wgpu::Texture, &wgpu::RenderPipeline, &wgpu::BindGroup); 3] = [
            (&f.bright, &self.bright, &f.show),
            (&f.across, &self.blur, &f.blur_across),
            (&f.down, &self.blur, &f.blur_down),
        ];
        for (target, pipeline, group) in passes {
            let view = target.create_view(&Default::default());
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bloom"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, group, &[]);
            pass.draw(0..3, 0..1);
        }
    }

    /// The scene, scaled by the white's ratio, onto the pass's target.
    pub fn show(&self, pass: &mut wgpu::RenderPass<'_>, f: &ExposeFrame) {
        pass.set_pipeline(&self.expose);
        pass.set_bind_group(0, &f.show, &[]);
        pass.draw(0..3, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each template's holes are filled, once, and no other brace pair
    /// names a constant.
    #[test]
    fn the_expose_templates_holes_are_filled() {
        for (template, holes, filled) in [
            (LUMINANCE_SHADER, &LUMINANCE_HOLES[..], luminance_shader()),
            (EXPOSE_SHADER, &EXPOSE_HOLES[..], expose_shader()),
        ] {
            for h in holes {
                assert!(template.contains(&format!("{{{h}}}")), "{h}");
            }
            assert!(
                !filled
                    .chars()
                    .zip(filled.chars().skip(1))
                    .any(|(a, b)| a == '{' && b.is_ascii_alphabetic()),
                "a hole the template names and the code does not fill"
            );
        }
    }
}
