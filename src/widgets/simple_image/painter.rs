use eframe::{
    egui_wgpu::{wgpu, RenderState},
    wgpu::{
        BindGroup, Buffer, CommandEncoderDescriptor, ComputePassDescriptor, Queue, TextureFormat,
        TextureView,
    },
};
use serde::{Deserialize, Serialize};
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, BufferDescriptor, BufferUsages, ComputePass, ComputePipeline,
    ComputePipelineDescriptor, PipelineLayoutDescriptor, ShaderModuleDescriptor, ShaderSource,
    ShaderStages, StorageTextureAccess, Texture, TextureDescriptor, TextureUsages,
    TextureViewDescriptor, TextureViewDimension,
};

unsafe fn as_raw_bytes<T>(x: &T) -> &[u8] {
    std::slice::from_raw_parts(x as *const T as *const u8, std::mem::size_of::<T>())
}

#[repr(C, align(16))]
#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Settings {
    time: f32,
}

impl Settings {
    fn descriptor<'a>() -> BufferDescriptor<'a> {
        BufferDescriptor {
            label: None,
            size: std::mem::size_of::<Self>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }
    }
}

#[derive(Debug)]
pub struct Painter {
    pipeline: ComputePipeline,
    texture: Texture,
    uniforms: Buffer,
    bind_group: BindGroup,
}

impl Painter {
    pub fn new(rc: &RenderState, width: u32, height: u32) -> Option<Self> {
        let texture = rc.device.create_texture(&TextureDescriptor {
            label: Some("simple_image output texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb], // maybe add srgb?
        });

        // Memory layout for the compute shader
        // There's just the one output texture
        let layout = rc
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("simple_image compute shader bind group layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            view_dimension: TextureViewDimension::D2,
                            access: StorageTextureAccess::WriteOnly,
                            format: texture.format(),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let uniforms = rc.device.create_buffer(&Settings::descriptor());

        let bind_group = rc.device.create_bind_group(&BindGroupDescriptor {
            label: Some("simple_image bind group"),
            layout: &layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.create_view(
                        &TextureViewDescriptor {
                            label: Some("simple_image texture view"),
                            ..Default::default()
                        },
                    )),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: uniforms.as_entire_binding(),
                },
            ],
        });

        let module = &rc.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("My Painter shader module"),
            source: ShaderSource::Wgsl(include_str!("compute.wgsl").into()),
        });

        let pipeline = rc
            .device
            .create_compute_pipeline(&ComputePipelineDescriptor {
                label: Some("simple_image compute pipeline"),
                layout: Some(
                    &rc.device.create_pipeline_layout(&PipelineLayoutDescriptor {
                        label: Some("simple_image pipeline layout"),
                        bind_group_layouts: &[&layout],
                        push_constant_ranges: &[],
                    }),
                ),
                module,
                entry_point: "main",
            });

        Some(Self {
            pipeline,
            texture,
            bind_group,
            uniforms,
        })
    }

    pub fn update(&self, queue: &Queue, settings: &Settings) {
        puffin::profile_function!();
        queue.write_buffer(&self.uniforms, 0, unsafe { as_raw_bytes(settings) });
    }

    pub fn create_texture_view(&self) -> TextureView {
        self.texture.create_view(&TextureViewDescriptor {
            label: Some("simple_image texture view"),
            format: Some(TextureFormat::Rgba8UnormSrgb),
            ..Default::default()
        })
    }

    pub fn compute<'rp>(&'rp self, pass: &mut ComputePass<'rp>) {
        puffin::profile_function!();
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_pipeline(&self.pipeline);
        let extent = self.texture.size();
        pass.dispatch_workgroups(extent.width, extent.height, 1);
    }

    pub fn oneshot(&self, rc: &RenderState) {
        let mut encoder = rc.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("simple_image oneshot commands"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("simple_image oneshot compute pass begin"),
            });
            self.compute(&mut pass);
        }
        rc.queue.submit([encoder.finish()].into_iter());
    }
}
