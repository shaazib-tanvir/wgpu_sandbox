use crate::pipeline::*;

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
                            model.mesh.positions[3*i],
                            model.mesh.positions[3*i + 1],
                            model.mesh.positions[3*i + 2],
                        ],
                        normal: [
                            model.mesh.normals[3*i],
                            model.mesh.normals[3*i + 1],
                            model.mesh.normals[3*i + 2],
                        ],
                        uv: [model.mesh.texcoords[2*i], model.mesh.texcoords[2*i + 1]],
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

pub struct Scene {
    pub objects: Vec<mesh::Object>,
    pub lights: Vec<mesh::PointLight>,
    pub camera: mesh::Camera,
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

// fn translation_transform(offset: cgmath::Vector3<f32>) -> cgmath::Matrix4<f32> {
//     return cgmath::Matrix4::from_cols(
//         cgmath::Vector4::new(0.0, 0.0, 0.0, offset.x),
//         cgmath::Vector4::new(0.0, 0.0, 0.0, offset.y),
//         cgmath::Vector4::new(0.0, 0.0, 0.0, offset.z),
//         cgmath::Vector4::new(0.0, 0.0, 0.0, 1.0),
//     );
// }

impl Scene {
    pub fn new(aspect: f32, camera_position: cgmath::Point3<f32>) -> Self {
        let camera = mesh::Camera {
            position: camera_position.into(),
            view_proj: (perspective_transform(0.1, 5.0, aspect, 0.75)
                * cgmath::Matrix4::look_at_lh(
                    camera_position,
                    cgmath::Point3::new(0.0, 0.0, 0.0),
                    cgmath::Vector3::unit_y(),
                ))
            .into(),
            _padding: 0.0,
        };

        // (|| -> Result<crate::scene::Model, tobj::LoadError> {
        //     let models = tobj::load_obj_buf(
        //         &mut std::io::Cursor::new(include_bytes!($name)),
        //         &tobj::LoadOptions {
        //             triangulate: true,
        //             single_index: true,
        //             ignore_points: true,
        //             ignore_lines: true,
        //         },
        //         |_| Ok((vec![], ahash::AHashMap::new())),
        //     )?
        //     .0;
        //
        //     let mut vertices = Vec::new();
        //     let mut indices = Vec::new();
        //     for model in models {
        //         debug_assert!(model.mesh.positions.len() % 3 == 0);
        //         for i in 0..model.mesh.positions.len() / 3 {
        //             let vertex = crate::pipeline::mesh::Vertex {
        //                 position: [
        //                     model.mesh.positions[3 * i],
        //                     model.mesh.positions[3 * i + 1],
        //                     model.mesh.positions[3 * i + 2],
        //                 ],
        //                 normal: [
        //                     model.mesh.normals[3 * i],
        //                     model.mesh.normals[3 * i + 1],
        //                     model.mesh.normals[3 * i + 2],
        //                 ],
        //                 uv: [model.mesh.texcoords[2 * i], model.mesh.texcoords[2 * i + 1]],
        //             };
        //             vertices.push(vertex);
        //         }
        //         for index in model.mesh.indices {
        //             indices.push(index);
        //         }
        //     }
        //
        //     Ok(crate::scene::Model {
        //         vertex_buffer: vertices,
        //         index_buffer: indices,
        //     })
        // })()

        return Scene {
            objects: vec![mesh::Object {
                model: (cgmath::Matrix4::from_cols(
                           cgmath::Vector4::new(-1.0, 0.0, 0.0, 0.0),
                           cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
                           cgmath::Vector4::new(0.0, 0.0, 1.0, 0.0),
                           cgmath::Vector4::new(0.0, 0.0, 0.0, 1.0),
                       )).into(),
                metallic: 0.5,
                _padding: [0.0, 0.0, 0.0],
            }],
            lights: vec![mesh::PointLight {
                position: [0.0, 1., -2.],
                color: [1.0, 1.0, 1.0],
                strength: 5.0,
                _padding0: 0.0,
                _padding1: [0.0, 0.0, 0.0, 0.0],
            }],
            camera: camera,
        };
    }
}
