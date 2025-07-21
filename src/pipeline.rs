pub mod mesh;
use crate::scene::{InitData, Scene};

pub trait Pipeline: Sized {
    type E;

    fn new(
        init_data: &InitData,
        scene: &Scene,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) -> Result<Self, Self::E>;
    fn update(&self, scene: &Scene, device: &wgpu::Device, queue: &wgpu::Queue);
    fn draw(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
    );
}

pub fn create_uniform_buffer<T>(device: &wgpu::Device, count: Option<u64>) -> wgpu::Buffer {
    let buffer_descriptor = wgpu::BufferDescriptor {
        label: None,
        size: match count {
            None => std::mem::size_of::<T>() as u64,
            Some(count) => (std::mem::size_of::<T>() as u64) * count,
        },
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    };

    device.create_buffer(&buffer_descriptor)
}

pub fn create_storage_buffer<T>(device: &wgpu::Device, count: Option<u64>) -> wgpu::Buffer {
    let buffer_descriptor = wgpu::BufferDescriptor {
        label: None,
        size: match count {
            None => std::mem::size_of::<T>() as u64,
            Some(count) => (std::mem::size_of::<T>() as u64) * count,
        },
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    };

    device.create_buffer(&buffer_descriptor)
}
