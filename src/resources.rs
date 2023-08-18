use kira::sound::static_sound::StaticSoundData;
use kira::sound::static_sound::StaticSoundSettings;
use std::io::{BufReader, Cursor};
use wgpu::util::DeviceExt;

use crate::renderer::model;
use crate::renderer::texture;

const ASSETS_ROOT_PATH: &str = "assets";

pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    let path = std::path::Path::new(ASSETS_ROOT_PATH).join(file_name);
    println!("Loading (text): {:?}", path);
    let txt = std::fs::read_to_string(path)?;

    Ok(txt)
}

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    let path = std::path::Path::new(ASSETS_ROOT_PATH).join(file_name);
    println!("Loading (binary): {:?}", path);
    let data = std::fs::read(path)?;

    Ok(data)
}

pub async fn load_texture(
    file_name: &str,
    is_normal_map: bool,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<texture::Texture> {
    let data = load_binary(file_name).await?;

    texture::Texture::from_bytes(device, queue, &data, file_name, is_normal_map)
}

// TODO: support loading models without a normal map.
pub async fn load_model_obj(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<model::Model> {
    let file_path = std::path::Path::new(file_name);
    let path_root = file_path.parent().unwrap();

    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            let path = std::path::Path::new(&path_root).join(p);
            let path_str = path.to_str().unwrap();
            let mat_text = load_string(path_str).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    let mut materials = Vec::new();
    for m in obj_materials? {
        let path_diffuse = std::path::Path::new(&path_root).join(&m.diffuse_texture);
        let path_normal = std::path::Path::new(&path_root).join(&m.normal_texture);
        let diffuse_texture =
            load_texture(path_diffuse.to_str().unwrap(), false, device, queue).await?;
        let normal_texture =
            load_texture(path_normal.to_str().unwrap(), true, device, queue).await?;

        materials.push(model::Material::new(
            device,
            &m.name,
            diffuse_texture,
            normal_texture,
            layout,
        ));
    }

    let mut model_min_x = f32::MAX;
    let mut model_min_y = f32::MAX;
    let mut model_min_z = f32::MAX;
    let mut model_max_x = f32::MIN;
    let mut model_max_y = f32::MIN;
    let mut model_max_z = f32::MIN;

    let meshes = models
        .into_iter()
        .map(|m| {
            let mut vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| model::ModelVertex {
                    position: [
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ],
                    tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
                    normal: [
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ],
                    tangent: [0.0; 3],
                    bitangent: [0.0; 3],
                })
                .collect::<Vec<_>>();

            let indices = &m.mesh.indices;
            let mut triangles_included = vec![0; vertices.len()];

            let mut min_x = f32::MAX;
            let mut min_y = f32::MAX;
            let mut min_z = f32::MAX;
            let mut max_x = f32::MIN;
            let mut max_y = f32::MIN;
            let mut max_z = f32::MIN;

            // Calculate tangents and bitangets. We're going to use the triangles,
            // so we need to loop through the indices in chunks of 3
            // https://sotrh.github.io/learn-wgpu/intermediate/tutorial11-normals/#the-tangent-and-the-bitangent
            // Also take this opportunity to get min and max positions.
            for c in indices.chunks(3) {
                let v0 = vertices[c[0] as usize];
                let v1 = vertices[c[1] as usize];
                let v2 = vertices[c[2] as usize];

                let pos0: cgmath::Vector3<_> = v0.position.into();
                let pos1: cgmath::Vector3<_> = v1.position.into();
                let pos2: cgmath::Vector3<_> = v2.position.into();

                let uv0: cgmath::Vector2<_> = v0.tex_coords.into();
                let uv1: cgmath::Vector2<_> = v1.tex_coords.into();
                let uv2: cgmath::Vector2<_> = v2.tex_coords.into();

                // Calculate the edges of the triangle
                let delta_pos1 = pos1 - pos0;
                let delta_pos2 = pos2 - pos0;

                // This will give us a direction to calculate the
                // tangent and bitangent
                let delta_uv1 = uv1 - uv0;
                let delta_uv2 = uv2 - uv0;

                // Solving the following system of equations will
                // give us the tangent and bitangent.
                //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
                //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
                let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
                let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
                // We flip the bitangent to enable right-handed normal
                // maps with wgpu texture coordinate system
                let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

                // We'll use the same tangent/bitangent for each vertex in the triangle
                vertices[c[0] as usize].tangent =
                    (tangent + cgmath::Vector3::from(vertices[c[0] as usize].tangent)).into();
                vertices[c[1] as usize].tangent =
                    (tangent + cgmath::Vector3::from(vertices[c[1] as usize].tangent)).into();
                vertices[c[2] as usize].tangent =
                    (tangent + cgmath::Vector3::from(vertices[c[2] as usize].tangent)).into();
                vertices[c[0] as usize].bitangent =
                    (bitangent + cgmath::Vector3::from(vertices[c[0] as usize].bitangent)).into();
                vertices[c[1] as usize].bitangent =
                    (bitangent + cgmath::Vector3::from(vertices[c[1] as usize].bitangent)).into();
                vertices[c[2] as usize].bitangent =
                    (bitangent + cgmath::Vector3::from(vertices[c[2] as usize].bitangent)).into();

                // Used to average the tangents/bitangents
                triangles_included[c[0] as usize] += 1;
                triangles_included[c[1] as usize] += 1;
                triangles_included[c[2] as usize] += 1;

                // Get min/max positions
                min_x = f32::min(min_x, pos0.x);
                min_x = f32::min(min_x, pos1.x);
                min_x = f32::min(min_x, pos2.x);
                min_y = f32::min(min_y, pos0.y);
                min_y = f32::min(min_y, pos1.y);
                min_y = f32::min(min_y, pos2.y);
                min_z = f32::min(min_z, pos0.z);
                min_z = f32::min(min_z, pos1.z);
                min_z = f32::min(min_z, pos2.z);
                max_x = f32::max(max_x, pos0.x);
                max_x = f32::max(max_x, pos1.x);
                max_x = f32::max(max_x, pos2.x);
                max_y = f32::max(max_y, pos0.y);
                max_y = f32::max(max_y, pos1.y);
                max_y = f32::max(max_y, pos2.y);
                max_z = f32::max(max_z, pos0.z);
                max_z = f32::max(max_z, pos1.z);
                max_z = f32::max(max_z, pos2.z);
            }

            // Average the tangents/bitangents
            for (i, n) in triangles_included.into_iter().enumerate() {
                let denom = 1.0 / n as f32;
                let mut v = &mut vertices[i];
                v.tangent = (cgmath::Vector3::from(v.tangent) * denom).into();
                v.bitangent = (cgmath::Vector3::from(v.bitangent) * denom).into();
            }

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_name)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", file_name)),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            // Get model min/max positions
            model_min_x = f32::min(min_x, model_min_x);
            model_min_y = f32::min(min_y, model_min_y);
            model_min_z = f32::min(min_z, model_min_z);
            model_max_x = f32::max(max_x, model_max_x);
            model_max_y = f32::max(max_y, model_max_y);
            model_max_z = f32::max(max_z, model_max_z);

            model::Mesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),

                min_x,
                min_y,
                min_z,
                max_x,
                max_y,
                max_z,
            }
        })
        .collect::<Vec<_>>();

    Ok(model::Model {
        meshes,
        materials,
        min_x: model_min_x,
        min_y: model_min_y,
        min_z: model_min_z,
        max_x: model_max_x,
        max_y: model_max_y,
        max_z: model_max_z,
    })
}

// TODO: add streaming audio loading support
pub async fn load_static_sound_data(file_name: &str) -> anyhow::Result<StaticSoundData> {
    let path = std::path::Path::new(ASSETS_ROOT_PATH).join(file_name);
    println!("Loading (static sound data): {:?}", path);

    let sound_data = StaticSoundData::from_file(path, StaticSoundSettings::default())?;

    Ok(sound_data)
}
