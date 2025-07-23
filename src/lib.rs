use log::error;
use pollster::FutureExt;
use std::collections::HashMap;
use std::sync::Arc;
use std::time;
use thiserror::Error;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    keyboard::*,
    window::{Window, WindowId},
};

mod pipeline;
mod scene;

use crate::pipeline::Pipeline;
use crate::scene::{InitData, Scene};

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
}

impl<'window> RendererState<'window> {
    async fn new(
        window: Arc<Window>,
        scene: &Scene,
        init_data: &InitData,
    ) -> Result<RendererState<'window>, anyhow::Error> {
        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
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
        Ok(RendererState {
            window: window,
            surface: surface,
            device: device,
            queue: queue,
            surface_config: config,
            is_surface_configured: false,
            mesh_pipeline: mesh_pipeline.unwrap(),
            depth_texture: depth_texture,
            depth_texture_view: depth_texture_view,
        })
    }

    fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture_descriptor = wgpu::TextureDescriptor {
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

        let view_descriptor = wgpu::TextureViewDescriptor {
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
            let (depth_texture, depth_texture_view) =
                RendererState::create_depth_texture(&self.device, &self.surface_config);
            self.depth_texture = depth_texture;
            self.depth_texture_view = depth_texture_view;

            if scene.is_some() {
                let camera = &mut scene.unwrap().camera;
                let new_aspect = (self.window.inner_size().width as f32)
                    / (self.window.inner_size().height as f32);
                camera.update(
                    camera.fov,
                    new_aspect,
                    camera.near,
                    camera.far,
                    camera.speed,
                    camera.rot_rate,
                );
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
            .texture
            .create_view(&surface_view_descriptor);

        self.mesh_pipeline.update(scene, &self.device, &self.queue);
        self.mesh_pipeline.draw(
            &self.device,
            &mut encoder,
            &surface_view,
            &self.depth_texture_view,
        );
        let buffer = encoder.finish();
        self.queue.submit(vec![buffer]);
        surface_texture.present();
        Ok(())
    }
}

pub struct App<'window> {
    state: Option<RendererState<'window>>,
    scene: Option<Scene>,
    kmap: HashMap<PhysicalKey, bool>,
    mouse_movements: Vec<(f32, f32)>,
    delta: f32,
}

impl<'window> App<'window> {
    pub fn new() -> App<'window> {
        return App {
            state: None,
            scene: None,
            kmap: HashMap::new(),
            mouse_movements: Vec::new(),
            delta: 0.0069,
        };
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
                    let init_data = InitData {
                        models: vec![
                            load_model!("../assets/cube.obj").unwrap(),
                            load_model!("../assets/monkey.obj").unwrap(),
                        ],
                    };
                    let scene = Scene::new(
                        (window.inner_size().width as f32) / (window.inner_size().height as f32),
                        cgmath::Point3::new(0.0, 1.2, -3.0),
                    );
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

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        match event {
            winit::event::DeviceEvent::MouseMotion { delta } => {
                self.mouse_movements.push((delta.0 as f32, delta.1 as f32));
            }
            _ => {}
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: winit::event::WindowEvent,
    ) {
        let instant;
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
                return;
            }
            WindowEvent::RedrawRequested => {
                instant = time::Instant::now();
                if self.state.is_none() || self.scene.is_none() {
                    return;
                }

                self.scene.as_mut().unwrap().update(
                    &self.kmap,
                    &mut self.mouse_movements,
                    self.delta,
                );
                match self
                    .state
                    .as_ref()
                    .unwrap()
                    .render(&self.scene.as_ref().unwrap())
                {
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

                let current = time::Instant::now();
                let delta_duration = current.duration_since(instant);
                self.delta = delta_duration.as_secs_f32();
            }
            WindowEvent::Resized(new_size) => {
                if self.state.is_none() {
                    return;
                }

                let state = self.state.as_mut().unwrap();
                state.resize(new_size.width, new_size.height, self.scene.as_mut());
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => match event.state {
                winit::event::ElementState::Pressed => {
                    self.kmap.insert(event.physical_key.clone(), true);
                }
                winit::event::ElementState::Released => {
                    self.kmap.insert(event.physical_key.clone(), false);
                }
            },
            _ => (),
        }
    }
}
