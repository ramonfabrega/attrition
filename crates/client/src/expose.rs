//! **The tone map's white, taken from the frame** (item 4212;
//! `docs/RENDERER.md` §3.21). `ps_tonemap.hlsl` scales the whole HDR frame
//! by `1 / tonemap(max(avgLuminance, gMinLuminance) · gWhitePoint)`, the
//! mean the luminance target's mip 10 holds (`ps_luminance.hlsl`). The
//! client's draws are each the tone map's output at the floor
//! (`MIN_LUMINANCE`), so the frame is drawn off screen, its mean HDR
//! luminance taken through the curve's inverse on the GPU
//! (`shaders/luminance.wgsl`), and the frame shown scaled by the white's
//! ratio (`shaders/expose.wgsl`) before the overlay.

/// The luminance pass's template.
pub(crate) const LUMINANCE_SHADER: &str = include_str!("../shaders/luminance.wgsl");
/// The expose pass's template.
pub(crate) const EXPOSE_SHADER: &str = include_str!("../shaders/expose.wgsl");

/// The names [`LUMINANCE_SHADER`] leaves a hole for, each filled once.
#[cfg(test)]
pub(crate) const LUMINANCE_HOLES: [&str; 5] = ["EXPOSURE", "white", "LUMA_R", "LUMA_G", "LUMA_B"];
/// The names [`EXPOSE_SHADER`] leaves a hole for.
#[cfg(test)]
pub(crate) const EXPOSE_HOLES: [&str; 2] = ["MIN_LUMINANCE", "WHITE_POINT"];

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
    use render::silhouette::{MIN_LUMINANCE, WHITE_POINT};
    EXPOSE_SHADER
        .replace("{MIN_LUMINANCE}", &format!("{MIN_LUMINANCE:?}"))
        .replace("{WHITE_POINT}", &format!("{WHITE_POINT:?}"))
}

/// The passes and their layouts, built once.
pub(crate) struct Expose {
    luminance: wgpu::ComputePipeline,
    mean: wgpu::ComputePipeline,
    expose: wgpu::RenderPipeline,
    compute_layout: wgpu::BindGroupLayout,
    expose_layout: wgpu::BindGroupLayout,
}

/// One frame's: the off-screen scene the frame is drawn into, and the
/// bind group the expose pass reads it and its mean through.
pub(crate) struct ExposeFrame {
    pub scene: wgpu::Texture,
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
            entries: &[texture(0), storage(1, true, wgpu::ShaderStages::FRAGMENT)],
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
        Expose {
            luminance: compute("cs_luminance"),
            mean: compute("cs_mean"),
            expose,
            compute_layout,
            expose_layout,
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
            ],
        });
        ExposeFrame {
            scene,
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
