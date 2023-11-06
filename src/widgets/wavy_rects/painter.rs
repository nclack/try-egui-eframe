use eframe::{
    egui_wgpu::{self, RenderState},
    wgpu::Queue,
};
use egui_wgpu::wgpu;
use log::trace;
use serde::{Deserialize, Serialize};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendState, Buffer, BufferBindingType, BufferDescriptor,
    BufferUsages, ColorTargetState, ColorWrites, Face, FragmentState, FrontFace, IndexFormat,
    MultisampleState, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology,
    RenderPass, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource,
    ShaderStages, VertexAttribute, VertexBufferLayout, VertexState, VertexStepMode,
};

unsafe fn as_u8_slice<T>(x: &[T]) -> &[u8] {
    std::slice::from_raw_parts(x.as_ptr() as *const u8, std::mem::size_of_val(x))
}

unsafe fn as_raw_bytes<T>(x: &T) -> &[u8] {
    std::slice::from_raw_parts(x as *const T as *const u8, std::mem::size_of::<T>())
}

#[repr(C)]
pub struct Vertex {
    pub xyz: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex {
    const ATTRS: [VertexAttribute; 2] = vertex_attr_array![
        0 => Float32x3,
        1 => Float32x2
    ];

    fn layout<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as _,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRS,
        }
    }
}

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct RectPainterSettings {
    pub edge: [f32; 4],
    pub fill: [f32; 4],
    pub line_width_px: f32,
    pub corner_radius_px: f32,
}

impl RectPainterSettings {
    fn descriptor<'a>() -> BufferDescriptor<'a> {
        BufferDescriptor {
            label: None,
            size: std::mem::size_of::<Self>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }
    }
}

impl Default for RectPainterSettings {
    fn default() -> Self {
        Self {
            edge: [0.0, 0.0, 0.0, 1.0],
            fill: [1.0, 1.0, 1.0, 1.0],
            line_width_px: 2.0,
            corner_radius_px: 0.0,
        }
    }
}

pub struct RectPainter {
    pipeline: RenderPipeline,
    bind_group: BindGroup,
    uniforms: Buffer,
    vertices: Buffer,
    vertex_count: usize,
    indexes: Buffer,
    index_count: usize,
}

impl RectPainter {
    pub(crate) fn new(rc: &RenderState) -> Self {
        // Memory layout for the painter
        let layout = rc
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("my painter bind group layout"),
                entries: &[
                    // Color
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT | ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let uniforms = rc.device.create_buffer(&RectPainterSettings::descriptor());

        let bind_group = rc.device.create_bind_group(&BindGroupDescriptor {
            label: Some("My painter bind group"),
            layout: &layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniforms.as_entire_binding(),
            }],
        });

        let module = &rc.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("My Painter shader module"),
            source: ShaderSource::Wgsl(include_str!("painter.wgsl").into()),
        });

        let pipeline = rc.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("My Painter Render Pipeline"),
            layout: Some(
                &rc.device.create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some("My Painter Render Pipeline Layout"),
                    bind_group_layouts: &[&layout],
                    push_constant_ranges: &[],
                }),
            ),
            vertex: VertexState {
                module,
                entry_point: "vs",
                buffers: &[Vertex::layout()],
            },
            fragment: Some(FragmentState {
                module,
                entry_point: "fs",
                targets: &[Some(ColorTargetState {
                    format: rc.target_format,
                    blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Geometry buffers
        let vertices = rc.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Painter vertex buffer"),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            contents: &[0; 6000], // FIXME: reallocation?
        });

        let indexes = rc.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Painter index buffer"),
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            contents: &[0; 6000], // FIXME: reallocation?
        });

        Self {
            pipeline,
            bind_group,
            uniforms,
            vertices,
            vertex_count: 0,
            indexes,
            index_count: 0,
        }
    }

    pub fn set_geometry(&mut self, queue: &Queue, vertices: &[Vertex], indexes: &[u32]) {
        puffin::profile_function!();
        self.vertex_count = vertices.len();
        self.index_count = indexes.len();
        queue.write_buffer(&self.vertices, 0, unsafe { as_u8_slice(vertices) });
        queue.write_buffer(&self.indexes, 0, unsafe { as_u8_slice(indexes) });
    }

    pub fn set_uniforms(&self, queue: &Queue, settings: &RectPainterSettings) {
        puffin::profile_function!();
        queue.write_buffer(&self.uniforms, 0, unsafe { as_raw_bytes(settings) });
    }

    /// Set up the render pass for the frame.
    pub fn paint<'rp>(&'rp self, pass: &mut RenderPass<'rp>) {
        puffin::profile_function!();
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        trace!(
            "vertex count {} size {} {:?}",
            self.vertex_count,
            self.vertices.size(),
            self.vertices
        );
        pass.set_vertex_buffer(
            0,
            self.vertices
                .slice(..(std::mem::size_of::<Vertex>() * self.vertex_count) as u64),
        );
        trace!(
            "index count {} size {} {:?}",
            self.index_count,
            self.indexes.size(),
            self.indexes
        );
        pass.set_index_buffer(
            self.indexes
                .slice(..(std::mem::size_of::<u32>() * self.index_count) as u64),
            IndexFormat::Uint32,
        );
        pass.draw_indexed(0..self.index_count as u32, 0, 0..1);
    }
}
