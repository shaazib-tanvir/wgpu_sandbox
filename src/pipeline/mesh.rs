use super::{Pipeline, create_storage_buffer, create_uniform_buffer};
use std::iter::zip;

use crate::scene::{InitData, Scene};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLight {
    pub position: [f32; 3],
    pub _padding0: f32,
    pub color: [f32; 3],
    pub strength: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Object {
    pub model: [[f32; 4]; 4],
    pub metallic: f32,
    pub _padding: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera {
    pub position: [f32; 3],
    pub _padding: f32,
    pub view_proj: [[f32; 4]; 4],
}

impl Vertex {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x3,
            2 => Float32x2,
        ],
    };
}

pub struct Mesh {
    pipeline: wgpu::RenderPipeline,
    uniform_groups: Vec<wgpu::BindGroup>,
    storage_group: wgpu::BindGroup,
    lights_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    object_buffers: Vec<wgpu::Buffer>,
    vertex_buffers: Vec<wgpu::Buffer>,
    index_buffers: Vec<wgpu::Buffer>,
    index_lengths: Vec<u32>,
}

impl Pipeline for Mesh {
    type E = ();

    fn new(
        init_data: &InitData,
        scene: &Scene,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) -> Result<Self, ()> {
        let shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/mesh.wgsl"));
        let color_state_target = [Some(wgpu::ColorTargetState {
            format: config.format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::all(),
        })];

        let uniform_group_layout_descriptor = wgpu::BindGroupLayoutDescriptor {
            label: Some("Mesh Uniform Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        };
        let uniform_group_layout =
            device.create_bind_group_layout(&uniform_group_layout_descriptor);

        let lights_buffer =
            create_storage_buffer::<PointLight>(device, Some(scene.lights.len() as u64));
        let camera_buffer = create_uniform_buffer::<Camera>(device, None);
        let mut object_buffers = Vec::new();
        for _ in &scene.objects {
            let object_buffer = create_uniform_buffer::<Object>(device, None);
            object_buffers.push(object_buffer);
        }

        let mut uniform_groups = Vec::new();
        for object_buffer in &object_buffers {
            let uniform_group_descriptor = wgpu::BindGroupDescriptor {
                label: Some("Mesh Uniform Bind Group"),
                layout: &uniform_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            camera_buffer.as_entire_buffer_binding(),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(
                            object_buffer.as_entire_buffer_binding(),
                        ),
                    },
                ],
            };
            let uniform_group = device.create_bind_group(&uniform_group_descriptor);
            uniform_groups.push(uniform_group);
        }

        let storage_group_layout_descriptor = wgpu::BindGroupLayoutDescriptor {
            label: Some("Mesh Storage Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        };
        let storage_group_layout =
            device.create_bind_group_layout(&storage_group_layout_descriptor);

        let storage_group_descriptor = wgpu::BindGroupDescriptor {
            label: Some("Mesh Storage Bind Group"),
            layout: &storage_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(lights_buffer.as_entire_buffer_binding()),
            }],
        };
        let storage_group = device.create_bind_group(&storage_group_descriptor);

        let pipeline_layout_descriptor = wgpu::PipelineLayoutDescriptor {
            label: Some("Mesh Pipeline Layout"),
            bind_group_layouts: &[&uniform_group_layout, &storage_group_layout],
            push_constant_ranges: &[],
        };
        let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_descriptor);

        let mut vertex_buffers = Vec::new();
        for vertex_buffer in init_data.models.iter().map(|model| &model.vertex_buffer) {
            let buffer_descriptor = wgpu::BufferDescriptor {
                label: Some("Vertex Buffer"),
                size: (vertex_buffer.len() * std::mem::size_of::<Vertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            };
            let buffer = device.create_buffer(&buffer_descriptor);

            queue.write_buffer(&buffer, 0, bytemuck::cast_slice(vertex_buffer.as_slice()));
            queue.submit(vec![]);

            vertex_buffers.push(buffer);
        }

        let mut index_buffers = Vec::new();
        let mut index_lengths = Vec::new();
        for index_buffer in init_data.models.iter().map(|model| &model.index_buffer) {
            let buffer_descriptor = wgpu::BufferDescriptor {
                label: Some("Index Buffer"),
                size: (index_buffer.len() * std::mem::size_of::<u32>()) as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            };
            let buffer = device.create_buffer(&buffer_descriptor);

            queue.write_buffer(&buffer, 0, bytemuck::cast_slice(index_buffer.as_slice()));
            queue.submit(vec![]);

            index_buffers.push(buffer);
            index_lengths.push(index_buffer.len() as u32);
        }

        let depth_stencil_state = wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_compare: wgpu::CompareFunction::LessEqual,
            depth_write_enabled: true,
            stencil: wgpu::StencilState {
                ..Default::default()
            },
            bias: wgpu::DepthBiasState {
                ..Default::default()
            },
        };

        let light_count = scene.lights.len() as f64;
        let compilation_options = wgpu::PipelineCompilationOptions {
            constants: &[("0", light_count)],
            ..Default::default()
        };
        let pipeline_descriptor = wgpu::RenderPipelineDescriptor {
            label: Some("Mesh Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vert_main"),
                compilation_options: compilation_options.clone(),
                buffers: &[Vertex::LAYOUT],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(depth_stencil_state),
            multisample: wgpu::MultisampleState {
                count: 1,
                ..Default::default()
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("frag_main"),
                compilation_options: compilation_options.clone(),
                targets: &color_state_target,
            }),
            multiview: None,
            cache: None,
        };

        let pipeline = device.create_render_pipeline(&pipeline_descriptor);

        return Ok(Mesh {
            pipeline: pipeline,
            camera_buffer: camera_buffer,
            object_buffers: object_buffers,
            lights_buffer: lights_buffer,
            uniform_groups: uniform_groups,
            storage_group: storage_group,
            vertex_buffers: vertex_buffers,
            index_buffers: index_buffers,
            index_lengths: index_lengths,
        });
    }

    fn update(&self, scene: &Scene, _device: &wgpu::Device, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.lights_buffer,
            0,
            bytemuck::cast_slice(scene.lights.as_slice()),
        );
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::bytes_of(&scene.camera.mesh_camera),
        );
        for (object, object_buffer) in zip(
            scene.objects.iter().as_ref(),
            self.object_buffers.iter().as_ref(),
        ) {
            queue.write_buffer(object_buffer, 0, bytemuck::bytes_of(object));
        }
    }

    fn draw(
        &self,
        _: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
    ) {
        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: Some("Mesh Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.003,
                        g: 0.017,
                        b: 0.032,
                        a: 1.,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        };

        let mut render_pass = encoder.begin_render_pass(&render_pass_descriptor);
        render_pass.set_pipeline(&self.pipeline);
        for i in 0..self.vertex_buffers.len() {
            let vertex_buffer = self.vertex_buffers.get(i).unwrap();
            let index_buffer = self.index_buffers.get(i).unwrap();

            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_bind_group(0, self.uniform_groups.get(i), &[]);
            render_pass.set_bind_group(1, &self.storage_group, &[]);
            render_pass.draw_indexed(0..self.index_lengths.get(i).unwrap().clone(), 0, 0..1);
        }
    }
}
