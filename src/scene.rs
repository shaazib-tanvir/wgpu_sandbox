use crate::pipeline::*;
use crate::cache::{Cache, VecCache};

use cgmath::{InnerSpace, Quaternion, Rotation3, Transform};
use std::collections::HashMap;
use winit::keyboard::*;

pub struct InitData {
    pub models: Vec<Model>,
}

pub struct Model {
    pub vertex_buffer: Vec<mesh::Vertex>,
    pub index_buffer: Vec<u32>,
}

#[macro_export]
macro_rules! load_model {
    ($name:literal) => {
        (|| -> Result<crate::scene::Model, tobj::LoadError> {
            let models = tobj::load_obj_buf(
                &mut std::io::Cursor::new(include_bytes!($name)),
                &tobj::LoadOptions {
                    triangulate: true,
                    single_index: true,
                    ignore_points: true,
                    ignore_lines: true,
                },
                |_| Ok((vec![], ahash::AHashMap::new())),
            )?
            .0;

            let mut vertices = Vec::new();
            let mut indices = Vec::new();
            for model in models {
                debug_assert!(model.mesh.positions.len() % 3 == 0);
                for i in 0..model.mesh.positions.len() / 3 {
                    let vertex = crate::pipeline::mesh::Vertex {
                        position: [
                            model.mesh.positions[3 * i],
                            model.mesh.positions[3 * i + 1],
                            model.mesh.positions[3 * i + 2],
                        ],
                        normal: [
                            model.mesh.normals[3 * i],
                            model.mesh.normals[3 * i + 1],
                            model.mesh.normals[3 * i + 2],
                        ],
                        uv: [model.mesh.texcoords[2 * i], model.mesh.texcoords[2 * i + 1]],
                    };
                    vertices.push(vertex);
                }
                for index in model.mesh.indices {
                    indices.push(index);
                }
            }

            Ok(crate::scene::Model {
                vertex_buffer: vertices,
                index_buffer: indices,
            })
        })()
    };
}

pub struct Camera {
    pub mesh_camera: mesh::Camera,
    pub view: cgmath::Matrix4<f32>,
    pub projection: cgmath::Matrix4<f32>,
    pub position: cgmath::Point3<f32>,
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
    pub speed: f32,
    pub rot_rate: f32,
}

impl Camera {
    pub fn new(
        fov: f32,
        aspect: f32,
        near: f32,
        far: f32,
        direction: cgmath::Vector3<f32>,
        position: cgmath::Point3<f32>,
        speed: f32,
        rot_rate: f32,
    ) -> Camera {
        let target = cgmath::Point3::new(
            position.x + direction.x,
            position.y + direction.y,
            position.z + direction.z,
        );
        let view = cgmath::Matrix4::look_at_lh(position, target, cgmath::Vector3::unit_y());
        let projection = perspective_transform(near, far, aspect, fov);
        return Camera {
            mesh_camera: mesh::Camera {
                position: position.into(),
                _padding: 0.0,
                view_proj: (projection * view).into(),
            },
            view: view,
            projection: projection,
            position: position,
            fov: fov,
            aspect: aspect,
            near: near,
            far: far,
            speed: speed,
            rot_rate: rot_rate,
        };
    }

    pub fn update(
        &mut self,
        fov: f32,
        aspect: f32,
        near: f32,
        far: f32,
        speed: f32,
        rot_rate: f32,
    ) {
        self.projection = perspective_transform(near, far, aspect, fov);
        let mesh_camera = mesh::Camera {
            position: self.position.into(),
            _padding: 0.0,
            view_proj: (self.projection * self.view).into(),
        };
        self.mesh_camera = mesh_camera;
        self.near = near;
        self.far = far;
        self.fov = fov;
        self.aspect = aspect;
        self.speed = speed;
        self.rot_rate = rot_rate;
    }
}

pub struct Scene {
    pub objects: VecCache<mesh::Object>,
    pub point_lights: VecCache<mesh::PointLight>,
    pub directional_lights: VecCache<mesh::DirectionalLight>,
    pub camera: Cache<Camera>,
}

pub fn perspective_transform(near: f32, far: f32, aspect: f32, fov: f32) -> cgmath::Matrix4<f32> {
    let c = 1.0 / f32::tan(fov / 2.0);
    return cgmath::Matrix4::from_cols(
        cgmath::Vector4::new(c / aspect, 0.0, 0.0, 0.0),
        cgmath::Vector4::new(0.0, c, 0.0, 0.0),
        cgmath::Vector4::new(0.0, 0.0, far / (far - near), 1.0),
        cgmath::Vector4::new(0.0, 0.0, -(far * near) / (far - near), 0.0),
    );
}

impl Scene {
    pub fn new(aspect: f32, camera_position: cgmath::Point3<f32>) -> Self {
        let direction =
            cgmath::Vector3::new(-camera_position.x, -camera_position.y, -camera_position.z)
                .normalize();
        let camera = Camera::new(
            0.75,
            aspect,
            0.1,
            100.0,
            direction,
            camera_position,
            2.5,
            0.4,
        );

        return Scene {
            objects: VecCache::new(vec![
                mesh::Object {
                    model: (cgmath::Matrix4::from_translation(cgmath::Vector3::new(0.0, 1.0, 0.))
                        * cgmath::Matrix4::from_cols(
                            cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
                            cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
                            cgmath::Vector4::new(0.0, 0.0, -1.0, 0.0),
                            cgmath::Vector4::new(0.0, 0.0, 0.0, 1.0),
                        ))
                    .into(),
                    metallic: 0.5,
                    _padding: [0.0, 0.0, 0.0],
                },
                mesh::Object {
                    model: (cgmath::Matrix4::from_translation(cgmath::Vector3::new(1.0, 1.0, 2.))
                        * cgmath::Matrix4::from_cols(
                            cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
                            cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
                            cgmath::Vector4::new(0.0, 0.0, -1.0, 0.0),
                            cgmath::Vector4::new(0.0, 0.0, 0.0, 1.0),
                        ))
                    .into(),
                    metallic: 0.8,
                    _padding: [0.0, 0.0, 0.0],
                },
                mesh::Object {
                    model: (cgmath::Matrix4::from_scale(100.0) * cgmath::Matrix4::from_cols(
                            cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
                            cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
                            cgmath::Vector4::new(0.0, 0.0, -1.0, 0.0),
                            cgmath::Vector4::new(0.0, 0.0, 0.0, 1.0)))
                    .into(),
                    metallic: 0.0,
                    _padding: [0.0, 0.0, 0.0],
                }
            ]),
            point_lights: VecCache::new(vec![
                mesh::PointLight {
                    position: [0.0, 2.0, -2.0],
                    color: [1.0, 1.0, 1.0],
                    strength: 0.0,
                    _padding0: 0.0,
                },
            ]),
            directional_lights: VecCache::new(vec![
                mesh::DirectionalLight {
                    position: [0.0, 2.0, -2.0],
                    color: [1.0, 1.0, 1.0],
                    direction: [0.0, -0.707, 0.707],
                    strength: 5.0,
                    _padding0: 0.0,
                    _padding1: 0.0,
                }
            ]),
            camera: Cache::new(camera),
        };
    }

    fn check_key(kmap: &HashMap<PhysicalKey, bool>, code: KeyCode) -> bool {
        return kmap
            .get(&PhysicalKey::Code(code))
            .is_some_and(|pressed| pressed.clone());
    }

    fn extract_rotation<S: Copy>(matrix: &cgmath::Matrix4<S>) -> cgmath::Matrix3<S> {
        return cgmath::Matrix3::new(
            matrix.x.x, matrix.x.y, matrix.x.z, matrix.y.x, matrix.y.y, matrix.y.z, matrix.z.x,
            matrix.z.y, matrix.z.z,
        );
    }

    pub fn update(
        &mut self,
        kmap: &HashMap<PhysicalKey, bool>,
        mouse_movements: &mut Vec<(f32, f32)>,
        delta: f32,
    ) {
        let forward_pressed = Self::check_key(&kmap, KeyCode::KeyW);
        let backwards_pressed = Self::check_key(&kmap, KeyCode::KeyS);
        let right_pressed = Self::check_key(&kmap, KeyCode::KeyD);
        let left_pressed = Self::check_key(&kmap, KeyCode::KeyA);

        let forward_axis = ((forward_pressed as i32) - (backwards_pressed as i32)) as f32;
        let side_axis = ((right_pressed as i32) - (left_pressed as i32)) as f32;

        let displacement = delta
            * self.camera.value.speed
            * ((-forward_axis * cgmath::Vector3::unit_z())
                + (-side_axis * cgmath::Vector3::unit_x()));
        let mut total_movement = (0.0, 0.0);
        for movement in mouse_movements.iter() {
            total_movement = (movement.0 + total_movement.0, movement.1 + total_movement.1);
        }
        mouse_movements.clear();
        let local_rotation = Quaternion::from_axis_angle(
            cgmath::Vector3::unit_x(),
            cgmath::Rad(-self.camera.value.rot_rate * total_movement.1 * delta),
        );

        let world_y = <cgmath::Matrix3<f32> as Transform<cgmath::Point3<f32>>>::transform_vector(
            &Self::extract_rotation(&self.camera.value.view),
            cgmath::Vector3::unit_y(),
        );
        let world_y = world_y.normalize();
        let global_rotation = Quaternion::from_axis_angle(
            world_y,
            cgmath::Rad(-self.camera.value.rot_rate * total_movement.0 * delta),
        );

        self.camera.value.view = cgmath::Matrix4::from_translation(displacement)
            * Into::<cgmath::Matrix4<f32>>::into(local_rotation)
            * Into::<cgmath::Matrix4<f32>>::into(global_rotation)
            * self.camera.value.view;
        self.camera.value.mesh_camera.view_proj = (self.camera.value.projection * self.camera.value.view).into();
        self.camera.dirty = true;
    }
}
