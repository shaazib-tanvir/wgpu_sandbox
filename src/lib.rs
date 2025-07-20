use log::error;
use pollster::FutureExt;
use std::sync::Arc;
use thiserror::Error;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent, event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

mod pipeline;
mod scene;

use crate::scene::{Scene, InitData};
use crate::pipeline::{Pipeline};

#[derive(Error, Debug)]
enum FormatError {
    #[error("no valid format was supported by the surface")]
    NotFound,
}

struct RendererState<'window> {
    window: Arc<Window>,
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    mesh_pipeline: pipeline::mesh::Mesh,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
    // pipeline: wgpu::RenderPipeline,
    // vertex_buffer: wgpu::Buffer,
    // bind_group: wgpu::BindGroup,
}

// #[repr(C)]
// #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
// struct Vertex {
//     position: [f32; 3],
//     color: [f32; 3],
//     texture_coord: [f32; 2],
// }
//
// impl Vertex {
//     pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
//         array_stride: size_of::<Self>() as wgpu::BufferAddress,
//         step_mode: wgpu::VertexStepMode::Vertex,
//         attributes: &wgpu::vertex_attr_array![
//             0 => Float32x3,
//             1 => Float32x3,
//             2 => Float32x2,
//         ],
//     };
// }
//
impl<'window> RendererState<'window> {
    async fn new(window: Arc<Window>, scene: &Scene, init_data: &InitData) -> Result<RendererState<'window>, anyhow::Error> {
        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            #[cfg(debug_assertions)]
            flags: wgpu::InstanceFlags::debugging(),
            #[cfg(not(debug_assertions))]
            flags: wgpu::InstanceFlags::empty(),
            ..Default::default()
        };

        let instance = wgpu::Instance::new(&instance_descriptor);
        let surface = instance.create_surface(window.clone())?;

        let request_adapter_options = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            ..Default::default()
        };

        let adapter = instance.request_adapter(&request_adapter_options).await?;
        let device_descriptor = wgpu::DeviceDescriptor {
            label: Some("Device"),
            required_features: wgpu::Features::empty(),
            memory_hints: wgpu::MemoryHints::Performance,
            ..Default::default()
        };
        let (device, queue) = adapter.request_device(&device_descriptor).await?;
        let capabilities = surface.get_capabilities(&adapter);
        let surface_format = capabilities
            .formats
            .iter()
            .copied()
            .find(|format| format.is_srgb());

        let surface_format = match surface_format {
            Some(surface_format) => surface_format,
            None => {
                return Err(FormatError::NotFound.into());
            }
        };

        let config = wgpu::SurfaceConfiguration {
            present_mode: wgpu::PresentMode::AutoVsync,
            width: window.inner_size().width,
            height: window.inner_size().height,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            desired_maximum_frame_latency: 2,
            format: surface_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: vec![],
        };

        let (depth_texture, depth_texture_view) = Self::create_depth_texture(&device, &config);

        let mesh_pipeline = pipeline::mesh::Mesh::new(init_data, scene, &device, &queue, &config);
        // let image_data = include_bytes!("../assets/dark_mode.png");
        // let image = image::load_from_memory_with_format(image_data, image::ImageFormat::Png)?;
        // let image_rgba = image.to_rgba8();
        // let texture_descriptor = wgpu::TextureDescriptor {
        //     label: Some("Texture"),
        //     size: wgpu::Extent3d {
        //         width: image_rgba.width(),
        //         height: image_rgba.height(),
        //         ..Default::default()
        //     },
        //     mip_level_count: 1,
        //     sample_count: 1,
        //     dimension: wgpu::TextureDimension::D2,
        //     format: wgpu::TextureFormat::Rgba8UnormSrgb,
        //     usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        //     view_formats: &[],
        // };
        // let texture = device.create_texture(&texture_descriptor);
        // queue.write_texture(
        //     wgpu::TexelCopyTextureInfo {
        //         texture: &texture,
        //         mip_level: 0,
        //         origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
        //         aspect: wgpu::TextureAspect::All,
        //     },
        //     &image_rgba,
        //     wgpu::TexelCopyBufferLayout {
        //         offset: 0,
        //         bytes_per_row: Some(image_rgba.width() * 4),
        //         rows_per_image: Some(image_rgba.height()),
        //     },
        //     wgpu::Extent3d {
        //         width: image_rgba.width(),
        //         height: image_rgba.height(),
        //         depth_or_array_layers: 1,
        //     },
        // );
        // queue.submit(vec![]);
        //
        // let texture_view_descriptor = wgpu::TextureViewDescriptor {
        //     label: Some("Texture View"),
        //     format: Some(texture.format()),
        //     dimension: Some(wgpu::TextureViewDimension::D2),
        //     usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
        //     aspect: wgpu::TextureAspect::All,
        //     base_mip_level: 0,
        //     mip_level_count: Some(1),
        //     base_array_layer: 0,
        //     array_layer_count: None,
        // };
        // let texture_view = texture.create_view(&texture_view_descriptor);
        // let sampler_descriptor = wgpu::SamplerDescriptor {
        //     label: Some("Sampler"),
        //     address_mode_u: wgpu::AddressMode::Repeat,
        //     address_mode_v: wgpu::AddressMode::Repeat,
        //     compare: None,
        //     border_color: None,
        //     mag_filter: wgpu::FilterMode::Nearest,
        //     min_filter: wgpu::FilterMode::Nearest,
        //     mipmap_filter: wgpu::FilterMode::Nearest,
        //     ..Default::default()
        // };
        // let sampler = device.create_sampler(&sampler_descriptor);
        //
        // let bind_group_layout_descriptor = wgpu::BindGroupLayoutDescriptor {
        //     label: Some("Bind Group Layout"),
        //     entries: &[
        //         wgpu::BindGroupLayoutEntry {
        //             binding: 0,
        //             visibility: wgpu::ShaderStages::FRAGMENT,
        //             ty: wgpu::BindingType::Texture {
        //                 sample_type: wgpu::TextureSampleType::Float { filterable: false },
        //                 multisampled: false,
        //                 view_dimension: wgpu::TextureViewDimension::D2,
        //             },
        //             count: None,
        //         },
        //         wgpu::BindGroupLayoutEntry {
        //             binding: 1,
        //             visibility: wgpu::ShaderStages::FRAGMENT,
        //             ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
        //             count: None,
        //         },
        //     ],
        // };
        // let bind_group_layout = device.create_bind_group_layout(&bind_group_layout_descriptor);
        // let bind_group_descriptor = wgpu::BindGroupDescriptor {
        //     label: Some("Bind Group"),
        //     layout: &bind_group_layout,
        //     entries: &[
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: wgpu::BindingResource::TextureView(&texture_view),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 1,
        //             resource: wgpu::BindingResource::Sampler(&sampler),
        //         },
        //     ],
        // };
        // let bind_group = device.create_bind_group(&bind_group_descriptor);
        //
        // let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/triangle.wgsl"));
        //
        // let color_state_target = [Some(wgpu::ColorTargetState {
        //     format: config.format,
        //     blend: Some(wgpu::BlendState::REPLACE),
        //     write_mask: wgpu::ColorWrites::all(),
        // })];
        //
        // let pipeline_layout_descriptor = wgpu::PipelineLayoutDescriptor{
        //     label: Some("Pipeline Layout"),
        //     bind_group_layouts: &[
        //         &bind_group_layout
        //     ],
        //     push_constant_ranges: &[],
        // };
        // let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_descriptor);
        // let render_pipeline_descriptor = wgpu::RenderPipelineDescriptor {
        //     label: Some("Pipeline"),
        //     layout: Some(&pipeline_layout),
        //     vertex: wgpu::VertexState {
        //         module: &shader,
        //         entry_point: Some("vert_main"),
        //         compilation_options: Default::default(),
        //         buffers: &[Vertex::LAYOUT],
        //     },
        //     primitive: wgpu::PrimitiveState {
        //         topology: wgpu::PrimitiveTopology::TriangleList,
        //         strip_index_format: None,
        //         front_face: wgpu::FrontFace::Ccw,
        //         cull_mode: Some(wgpu::Face::Back),
        //         unclipped_depth: false,
        //         polygon_mode: wgpu::PolygonMode::Fill,
        //         conservative: false,
        //     },
        //     depth_stencil: None,
        //     multisample: wgpu::MultisampleState {
        //         count: 1,
        //         ..Default::default()
        //     },
        //     fragment: Some(wgpu::FragmentState {
        //         module: &shader,
        //         entry_point: Some("frag_main"),
        //         compilation_options: Default::default(),
        //         targets: &color_state_target,
        //     }),
        //     multiview: None,
        //     cache: None,
        // };
        //
        // let pipeline = device.create_render_pipeline(&render_pipeline_descriptor);
        //
        // const VERTICES: &[Vertex] = &[
        //     Vertex {
        //         position: [0.0, 0.5, 0.0],
        //         color: [1.0, 0.0, 0.0],
        //         texture_coord: [0.0, 0.5],
        //     },
        //     Vertex {
        //         position: [-0.5, -0.5, 0.0],
        //         color: [0.0, 1.0, 0.0],
        //         texture_coord: [1.0, 0.0],
        //     },
        //     Vertex {
        //         position: [0.5, -0.5, 0.0],
        //         color: [0.0, 0.0, 1.0],
        //         texture_coord: [1.0, 1.0],
        //     },
        // ];
        // let vertex_buffer_descriptor = wgpu::BufferDescriptor {
        //     label: Some("Vertex Buffer 1"),
        //     size: (VERTICES.len() * size_of::<Vertex>()) as u64,
        //     mapped_at_creation: true,
        //     usage: wgpu::BufferUsages::VERTEX,
        // };
        // let vertex_buffer = device.create_buffer(&vertex_buffer_descriptor);
        // {
        //     let mut mapped_vertex_buffer =
        //         vertex_buffer.get_mapped_range_mut(0..vertex_buffer.size());
        //     mapped_vertex_buffer.copy_from_slice(bytemuck::cast_slice(VERTICES));
        // }
        // vertex_buffer.unmap();
        //
        Ok(RendererState {
            window: window,
            surface: surface,
            device: device,
            queue: queue,
            surface_config: config,
            is_surface_configured: false,
            mesh_pipeline: mesh_pipeline.unwrap(),
            depth_texture: depth_texture,
            depth_texture_view: depth_texture_view
            // pipeline: pipeline,
            // vertex_buffer: vertex_buffer,
            // bind_group: bind_group,
        })
    }

    fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> (wgpu::Texture, wgpu::TextureView) {
        let texture_descriptor = wgpu::TextureDescriptor{
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                ..Default::default()
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };
        let texture = device.create_texture(&texture_descriptor);

        let view_descriptor = wgpu::TextureViewDescriptor{
            label: Some("Depth Texture View"),
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
            usage: None,
            aspect: wgpu::TextureAspect::DepthOnly,
            dimension: Some(wgpu::TextureViewDimension::D2),
            format: Some(wgpu::TextureFormat::Depth32Float),
        };
        let view = texture.create_view(&view_descriptor);

        return (texture, view);
    }

    fn resize(&mut self, width: u32, height: u32, scene: Option<&mut Scene>) {
        if width > 0 && height > 0 {
            self.is_surface_configured = true;
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
            let (depth_texture, depth_texture_view) = RendererState::create_depth_texture(&self.device, &self.surface_config);
            self.depth_texture = depth_texture;
            self.depth_texture_view = depth_texture_view;

            if scene.is_some() {
                scene.unwrap().camera.view_proj = (crate::scene::perspective_transform(0.1, 5.0, (self.window.inner_size().width as f32) / (self.window.inner_size().height as f32), 0.75)
                * cgmath::Matrix4::look_at_lh(
                    cgmath::Point3::new(0.0, 1.2, -3.0),
                    cgmath::Point3::new(0.0, 0.0, 0.0),
                    cgmath::Vector3::unit_y(),
                )).into();
            }
        }
    }

    fn render(&self, scene: &Scene) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        if !self.is_surface_configured {
            return Ok(());
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });

        let surface_texture = self.surface.get_current_texture()?;
        let surface_view_descriptor = wgpu::TextureViewDescriptor {
            label: Some("Surface Texture View"),
            format: Some(self.surface_config.format),
            dimension: Some(wgpu::TextureViewDimension::D2),
            usage: Some(wgpu::TextureUsages::RENDER_ATTACHMENT),
            aspect: wgpu::TextureAspect::All,
            ..Default::default()
        };
        let surface_view = surface_texture
            .texture.create_view(&surface_view_descriptor);

        self.mesh_pipeline.update(scene, &self.device, &self.queue);
        self.mesh_pipeline.draw(&self.device, &mut encoder, &surface_view, &self.depth_texture_view);
        //
        // {
        //     let render_pass_descriptor = wgpu::RenderPassDescriptor {
        //         label: Some("Render Pass"),
        //         color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        //             view: &surface_view,
        //             resolve_target: None,
        //             ops: wgpu::Operations {
        //                 load: wgpu::LoadOp::Clear(wgpu::Color {
        //                     r: 0.003,
        //                     g: 0.017,
        //                     b: 0.032,
        //                     a: 1.,
        //                 }),
        //                 store: wgpu::StoreOp::Store,
        //             },
        //             depth_slice: None,
        //         })],
        //         depth_stencil_attachment: None,
        //         timestamp_writes: None,
        //         occlusion_query_set: None,
        //     };
        //
        //     let mut render_pass = encoder.begin_render_pass(&render_pass_descriptor);
        //     render_pass.set_pipeline(&self.pipeline);
        //     render_pass.set_bind_group(0, &self.bind_group, &[]);
        //     render_pass
        //         .set_vertex_buffer(0, self.vertex_buffer.slice(0..self.vertex_buffer.size()));
        //     render_pass.draw(0..3, 0..1);
        // }
        //
        let buffer = encoder.finish();
        self.queue.submit(vec![buffer]);
        surface_texture.present();
        Ok(())
    }
}

pub struct App<'window> {
    state: Option<RendererState<'window>>,
    scene: Option<Scene>,
}

impl<'window> App<'window> {
    pub fn new() -> App<'window> {
        return App { state: None, scene: None };
    }
}

impl<'window> ApplicationHandler for App<'window> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let attrs = Window::default_attributes()
                .with_title("WGPU Sandbox")
                .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
                .with_maximized(true)
                .with_resizable(true);
            match event_loop.create_window(attrs) {
                Err(err) => {
                    error!("failed to create window {}", err);
                    event_loop.exit();
                }
                Ok(window) => {
                    let init_data = InitData{
                        models: vec![load_model!("../assets/cube.obj").unwrap()]
                    };
                    let scene = Scene::new((window.inner_size().width as f32) / (window.inner_size().height as f32), cgmath::Point3::new(0.0, 1.2, -3.0));
                    let state = RendererState::new(Arc::new(window), &scene, &init_data).block_on();
                    match state {
                        Ok(state) => {
                            self.state = Some(state);
                            self.scene = Some(scene);
                        }
                        Err(err) => {
                            error!("failed to initialize renderer state {}", err);
                            event_loop.exit();
                        }
                    }
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
                return;
            }
            WindowEvent::RedrawRequested => {
                if self.state.is_none() || self.scene.is_none() {
                    return;
                }

                match self.state.as_ref().unwrap().render(&self.scene.as_ref().unwrap()) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                        let state = self.state.as_mut().unwrap();
                        state.resize(
                            state.window.inner_size().width,
                            state.window.inner_size().height,
                            self.scene.as_mut(),
                        );
                    }
                    Err(err) => {
                        error!("an error occured while rendering: {}", err);
                    }
                }
            }
            WindowEvent::Resized(new_size) => {
                if self.state.is_none() {
                    return;
                }

                let state = self.state.as_mut().unwrap();
                state.resize(new_size.width, new_size.height, self.scene.as_mut());
            }
            _ => (),
        }
    }
}
