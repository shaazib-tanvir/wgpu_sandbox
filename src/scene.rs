use crate::pipeline::*;
use cgmath::InnerSpace;
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
    pub position: cgmath::Point3<f32>,
    pub direction: cgmath::Vector3<f32>,
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
    pub speed: f32,
}

impl Camera {
    pub fn new(fov: f32, aspect: f32, near: f32, far: f32, direction: cgmath::Vector3<f32>, position: cgmath::Point3<f32>, speed: f32) -> Camera {
        let target = cgmath::Point3::new(position.x + direction.x, position.y + direction.y, position.z + direction.z);
        return Camera {
            mesh_camera: mesh::Camera {
                position: position.into(),
                _padding: 0.0,
                view_proj: (perspective_transform(near, far, aspect, fov) * cgmath::Matrix4::look_at_lh(position, target, cgmath::Vector3::unit_y())).into(),
            },
            position: position,
            direction: direction,
            fov: fov,
            aspect: aspect,
            near: near,
            far: far,
            speed: speed,
        }
    }

    pub fn update_camera(&mut self, fov: f32, aspect: f32, near: f32, far: f32, direction: cgmath::Vector3<f32>, position: cgmath::Point3<f32>, speed: f32) {
        let target = cgmath::Point3::new(position.x + direction.x, position.y + direction.y, position.z + direction.z);
        let mesh_camera = mesh::Camera {
            position: position.into(),
            _padding: 0.0,
            view_proj: (perspective_transform(near, far, aspect, fov) * cgmath::Matrix4::look_at_lh(position, target, cgmath::Vector3::unit_y())).into(),
        };
        self.mesh_camera = mesh_camera;
        self.near = near;
        self.far = far;
        self.fov = fov;
        self.aspect = aspect;
        self.position = position;
        self.direction = direction;
        self.speed = speed;
    }

    pub fn move_camera(&mut self, new_position: cgmath::Point3<f32>) {
        self.update_camera(self.fov, self.aspect, self.near, self.far, self.direction, new_position, self.speed);
    }
}

pub struct Scene {
    pub objects: Vec<mesh::Object>,
    pub lights: Vec<mesh::PointLight>,
    pub camera: Camera,
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
        let direction = cgmath::Vector3::new(-camera_position.x, -camera_position.y, -camera_position.z).normalize();
        let camera = Camera::new(0.75, aspect, 0.1, 10.0, direction, camera_position, 2.5);

        return Scene {
            objects: vec![mesh::Object {
                model: (cgmath::Matrix4::from_cols(
                    cgmath::Vector4::new(-1.0, 0.0, 0.0, 0.0),
                    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
                    cgmath::Vector4::new(0.0, 0.0, 1.0, 0.0),
                    cgmath::Vector4::new(0.0, 0.0, 0.0, 1.0),
                ))
                .into(),
                metallic: 0.5,
                _padding: [0.0, 0.0, 0.0],
            }, mesh::Object {
                model: (cgmath::Matrix4::from_translation(cgmath::Vector3::new(1.0, 0.2, 2.)) * cgmath::Matrix4::from_cols(
                    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
                    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
                    cgmath::Vector4::new(0.0, 0.0, -1.0, 0.0),
                    cgmath::Vector4::new(0.0, 0.0, 0.0, 1.0),
                ))
                .into(),
                metallic: 0.8,
                _padding: [0.0, 0.0, 0.0],
            }],
            lights: vec![
                mesh::PointLight {
                    position: [1.0, 2.0, -1.0],
                    color: [1.0, 1.0, 0.0],
                    strength: 2.5,
                    _padding0: 0.0,
                },
                mesh::PointLight {
                    position: [-1.0, 2.0, -2.0],
                    color: [0.0, 1.0, 1.0],
                    strength: 3.5,
                    _padding0: 0.0,
                },
            ],
            camera: camera,
        };
    }

    fn check_key(kmap: &HashMap<PhysicalKey, bool>, code: KeyCode) -> bool {
        return kmap.get(&PhysicalKey::Code(code)).is_some_and(|pressed| pressed.clone());
    }

    pub fn update(&mut self, kmap: &HashMap<PhysicalKey, bool>, delta: f32) {
        let forward_pressed = Self::check_key(&kmap, KeyCode::KeyW);
        let backwards_pressed = Self::check_key(&kmap, KeyCode::KeyS);
        let right_pressed = Self::check_key(&kmap, KeyCode::KeyD);
        let left_pressed = Self::check_key(&kmap, KeyCode::KeyA);

        let forward_axis = ((forward_pressed as i32) - (backwards_pressed as i32)) as f32;
        let side_axis = ((right_pressed as i32) - (left_pressed as i32)) as f32;
        let right_direction = cgmath::Vector3::unit_y().cross(self.camera.direction);
        let displacement = delta * self.camera.speed * (forward_axis * self.camera.direction + right_direction * side_axis);
        let new_position = cgmath::Point3::new(displacement.x + self.camera.position.x, displacement.y + self.camera.position.y, displacement.z + self.camera.position.z);
        self.camera.move_camera(new_position);
    }
}
