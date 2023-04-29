use gltf::Gltf;
use kira::sound::static_sound::StaticSoundData;
use kira::sound::static_sound::StaticSoundSettings;
use std::io::{BufReader, Cursor};
use wgpu::util::DeviceExt;

use crate::renderer::model;
use crate::renderer::texture;

const ASSETS_ROOT_PATH: &str = "assets";

pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join(ASSETS_ROOT_PATH)
        .join(file_name);
    println!("Loading (text): {:?}", path);
    let txt = std::fs::read_to_string(path)?;

    Ok(txt)
}

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("assets")
        .join(file_name);
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

            // Calculate tangents and bitangets. We're going to use the triangles,
            // so we need to loop through the indices in chunks of 3
            // https://sotrh.github.io/learn-wgpu/intermediate/tutorial11-normals/#the-tangent-and-the-bitangent
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

            model::Mesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            }
        })
        .collect::<Vec<_>>();

    Ok(model::Model { meshes, materials })
}

pub async fn load_model_gltf(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<model::Model> {
    let gltf_text = load_string(file_name).await?;
    let gltf_cursor = Cursor::new(gltf_text);
    let gltf_reader = BufReader::new(gltf_cursor);
    let gltf = Gltf::from_reader(gltf_reader)?;

    let path = std::path::Path::new(file_name);
    let parent_path = path.parent().unwrap();

    // Load buffers
    let mut buffer_data = Vec::new();
    for buffer in gltf.buffers() {
        match buffer.source() {
            // TODO: support loading bin buffer directly
            gltf::buffer::Source::Bin => {
                // if let Some(blob) = gltf.blob.as_deref() {
                //     buffer_data.push(blob.into());
                //     println!("Found a bin, saving");
                // };
            }
            gltf::buffer::Source::Uri(uri) => {
                let uri = parent_path.join(uri);
                let bin = load_binary(uri.to_str().unwrap()).await?;
                buffer_data.push(bin);
            }
        }
    }

    let mut materials = Vec::new();
    for material in gltf.materials() {
        println!("Looping thru materials");
        let pbr = material.pbr_metallic_roughness();

        let diffuse_texture_source = &pbr
            .base_color_texture()
            .map(|tex| {
                println!("Grabbing diffuse tex");
                dbg!(&tex.texture().source());
                tex.texture().source().source()
            })
            .expect("texture");

        let diffuse_texture = match diffuse_texture_source {
            gltf::image::Source::View { view, mime_type } => texture::Texture::from_bytes(
                device,
                queue,
                &buffer_data[view.buffer().index()],
                file_name,
                false,
            )
            .expect("Couldn't load diffuse"),

            gltf::image::Source::Uri { uri, mime_type: _ } => {
                let uri = parent_path.join(uri);
                load_texture(uri.to_str().unwrap(), false, device, queue).await?
            }
        };

        // TODO: get actual normal texture
        let normal_texture_source = &pbr
            .base_color_texture()
            .map(|tex| {
                println!("Grabbing diffuse tex");
                dbg!(&tex.texture().source());
                tex.texture().source().source()
            })
            .expect("texture");

        let normal_texture = match normal_texture_source {
            gltf::image::Source::View { view, mime_type: _ } => texture::Texture::from_bytes(
                device,
                queue,
                &buffer_data[view.buffer().index()],
                file_name,
                true,
            )
            .expect("Couldn't load normal texture"),

            gltf::image::Source::Uri { uri, mime_type } => {
                let uri = parent_path.join(uri);
                load_texture(uri.to_str().unwrap(), true, device, queue).await?
            }
        };

        let name = material.name().unwrap_or("Default Material");

        materials.push(model::Material::new(
            device,
            &name,
            diffuse_texture,
            normal_texture,
            layout,
        ));
    }

    let mut meshes = Vec::new();

    for scene in gltf.scenes() {
        for node in scene.nodes() {
            println!("Node {}", node.index());
            // dbg!(node);

            let mesh = node.mesh().expect("Got mesh");
            let primitives = mesh.primitives();
            primitives.for_each(|primitive| {
                // dbg!(primitive);
                // Grab the material data (like texture)
                let material = primitive.material().index();
                // The index buffer data
                let indices = primitive.indices().expect("got indices");

                let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));

                let mut vertices = Vec::new();
                if let Some(vertex_attribute) = reader.read_positions() {
                    vertex_attribute.for_each(|vertex| {
                        dbg!(vertex);
                        vertices.push(model::ModelVertex {
                            position: vertex,
                            tex_coords: Default::default(),
                            normal: Default::default(),
                            tangent: [0.0; 3],
                            bitangent: [0.0; 3],
                        })
                    });
                }
                if let Some(normal_attribute) = reader.read_normals() {
                    let mut normal_index = 0;
                    normal_attribute.for_each(|normal| {
                        dbg!(normal);
                        vertices[normal_index].normal = normal;

                        normal_index += 1;
                    });
                }
                if let Some(tex_coord_attribute) = reader.read_tex_coords(0).map(|v| v.into_f32()) {
                    let mut tex_coord_index = 0;
                    tex_coord_attribute.for_each(|tex_coord| {
                        dbg!(tex_coord);
                        vertices[tex_coord_index].tex_coords = tex_coord;

                        tex_coord_index += 1;
                    });
                }

                let mut indices = Vec::new();
                if let Some(indices_raw) = reader.read_indices() {
                    // dbg!(indices_raw);
                    indices.append(&mut indices_raw.into_u32().collect::<Vec<u32>>());
                }
                // dbg!(indices);

                // println!("{:#?}", &indices.expect("got indices").data_type());
                // println!("{:#?}", &indices.expect("got indices").index());
                // println!("{:#?}", &material);

                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Vertex Buffer", file_name)),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Index Buffer", file_name)),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

                meshes.push(model::Mesh {
                    name: file_name.to_string(),
                    vertex_buffer,
                    index_buffer,
                    num_elements: indices.len() as u32,
                    // material: m.mesh.material_id.unwrap_or(0),
                    material: 0,
                });
            });
        }
    }

    return Ok(model::Model { meshes, materials });
}

// TODO: add streaming audio loading support
pub async fn load_static_sound_data(file_name: &str) -> anyhow::Result<StaticSoundData> {
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("assets")
        .join(file_name);
    println!("Loading (static sound data): {:?}", path);

    let sound_data = StaticSoundData::from_file(path, StaticSoundSettings::default())?;

    Ok(sound_data)
}
