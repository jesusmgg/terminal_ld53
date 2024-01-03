use anyhow::Result;
use cgmath::{EuclideanSpace, Point3};
use wgpu::util::DeviceExt;

use crate::{
    game::{mesh_renderer::MeshInstancedRendererMgr, model::ModelMgr, transform::TransformMgr},
    renderer::{
        model::{self, ModelVertex},
        render_state::{self, RenderState},
    },
};

const MAX_INSTANCE_COUNT: usize = 128;
/// Maximum simultaneous collisions per instance.
const MAX_COLLISIONS: usize = 4;

/// Manages bounding boxes and collisions.
pub struct ColliderMgr {
    bounding_box_min: Vec<Point3<f32>>,
    bounding_box_max: Vec<Point3<f32>>,
    should_render_bounding_box: Vec<bool>,

    collider_type: Vec<ColliderType>,

    /// Whether the collider checks for collision with other colliders or not.
    is_collision_source: Vec<bool>,
    /// Whether the collider is checked for collision by other colliders or not.
    is_collision_target: Vec<bool>,

    /// Updated each frame
    pub colliding_indices: Vec<[isize; MAX_COLLISIONS]>,

    // References
    transform_i: Vec<usize>,
    model_i: Vec<Option<usize>>,
    bounding_box_model_i: Vec<usize>,
    bounding_box_mesh_renderer_i: Vec<usize>,
}

impl ColliderMgr {
    pub fn new() -> ColliderMgr {
        ColliderMgr {
            bounding_box_min: Vec::with_capacity(MAX_INSTANCE_COUNT),
            bounding_box_max: Vec::with_capacity(MAX_INSTANCE_COUNT),

            collider_type: Vec::with_capacity(MAX_INSTANCE_COUNT),

            is_collision_source: Vec::with_capacity(MAX_INSTANCE_COUNT),
            is_collision_target: Vec::with_capacity(MAX_INSTANCE_COUNT),

            colliding_indices: Vec::with_capacity(MAX_INSTANCE_COUNT),

            should_render_bounding_box: Vec::with_capacity(MAX_INSTANCE_COUNT),

            bounding_box_model_i: Vec::with_capacity(MAX_INSTANCE_COUNT),
            bounding_box_mesh_renderer_i: Vec::with_capacity(MAX_INSTANCE_COUNT),
            transform_i: Vec::with_capacity(MAX_INSTANCE_COUNT),
            model_i: Vec::with_capacity(MAX_INSTANCE_COUNT),
        }
    }

    pub async fn add_from_model(
        &mut self,
        model_i: usize,
        transform_i: usize,
        collider_type: ColliderType,
        is_collision_source: bool,
        is_collision_target: bool,
        render_bounding_box: bool,
        render_state: &RenderState,
        transform_mgr: &TransformMgr,
        model_mgr: &mut ModelMgr,
        mesh_renderer_mgr: &mut MeshInstancedRendererMgr,
    ) -> Result<usize> {
        let model = &model_mgr.model[model_i];

        let bbox_min = Point3 {
            x: model.min_x,
            y: model.min_y,
            z: model.min_z,
        };
        let bbox_max = Point3 {
            x: model.max_x,
            y: model.max_y,
            z: model.max_z,
        };

        self.bounding_box_min.push(bbox_min);
        self.bounding_box_max.push(bbox_max);

        self.collider_type.push(collider_type);

        self.is_collision_source.push(is_collision_source);
        self.is_collision_target.push(is_collision_target);

        self.should_render_bounding_box.push(render_bounding_box);

        self.colliding_indices.push([-1; MAX_COLLISIONS]);

        self.transform_i.push(transform_i);
        self.model_i.push(Some(model_i));

        let position = transform_mgr.position[transform_i];
        let rotation = transform_mgr.rotation[transform_i];
        let bbox_model = self
            .create_bounding_box_model(&bbox_min, &bbox_max, &render_state, mesh_renderer_mgr)
            .await;
        let bbox_model_i = model_mgr.add(
            bbox_model,
            format!("Collider bbox {}", self.len() - 1).as_str(),
        );
        let bbox_mesh_renderer_i =
            mesh_renderer_mgr.add(&render_state, bbox_model_i, position.to_vec(), rotation);
        self.bounding_box_model_i.push(bbox_model_i);
        self.bounding_box_mesh_renderer_i.push(bbox_mesh_renderer_i);

        let index = self.len() - 1;

        Ok(index)
    }

    pub fn update(&mut self, transform_mgr: &TransformMgr, model_mgr: &ModelMgr) {
        // OPTIMIZE: parallelize collision checks
        for index in 0..self.len() {
            self.colliding_indices[index] = [-1; MAX_COLLISIONS];
        }
        for index in 0..self.len() {
            if !self.is_collision_source[index] {
                continue;
            }
            self.colliding_indices[index] = self.check_collisions(index, transform_mgr, model_mgr)
        }
    }

    /// Checks and instance for collisions and returns colliding indexes.
    /// A -1 value in the returned array means no collision and should be ignored.
    fn check_collisions(
        &self,
        index: usize,
        transform_mgr: &TransformMgr,
        model_mgr: &ModelMgr,
    ) -> [isize; MAX_COLLISIONS] {
        let transform_i = self.transform_i[index];
        let position = transform_mgr.position[transform_i];

        let min_pos = self.get_translated_min_pos(index, position);
        let max_pos = self.get_translated_max_pos(index, position);

        // OPTIMIZE: check the other instance for an already ocurring collision with this instance.

        let mut collisions: [isize; MAX_COLLISIONS] = [-1; MAX_COLLISIONS];
        let mut collisions_found = 0;
        for other_index in 0..self.len() {
            if index == other_index {
                continue;
            }
            if !self.is_collision_target[other_index] {
                continue;
            }

            let other_transform_i = self.transform_i[other_index];
            let other_position = transform_mgr.position[other_transform_i];
            let other_min_pos = self.get_translated_min_pos(other_index, other_position);
            let other_max_pos = self.get_translated_max_pos(other_index, other_position);

            // Box collision check
            if min_pos.x <= other_max_pos.x
                && max_pos.x >= other_min_pos.x
                && min_pos.y <= other_max_pos.y
                && max_pos.y >= other_min_pos.y
                && min_pos.z <= other_max_pos.z
                && max_pos.z >= other_min_pos.z
            {
                let collider_type = &self.collider_type[index];
                let other_collider_type = &self.collider_type[other_index];
                let is_colliding = match (collider_type, other_collider_type) {
                    (ColliderType::Box, ColliderType::Box) => true,
                    (ColliderType::Box, ColliderType::Vertex) => {
                        self.check_collision_box_mesh(index, other_index, transform_mgr, model_mgr)
                    }
                    (ColliderType::Vertex, ColliderType::Box) => {
                        self.check_collision_box_mesh(other_index, index, transform_mgr, model_mgr)
                    }
                    (ColliderType::Vertex, ColliderType::Vertex) => todo!(),
                };
                if is_colliding {
                    collisions[collisions_found] = other_index as isize;
                    collisions_found += 1;
                }
            }

            if collisions_found >= MAX_COLLISIONS {
                break;
            }
        }

        collisions
    }

    /// index: box
    /// other_index: mesh
    fn check_collision_box_mesh(
        &self,
        index: usize,
        other_index: usize,
        transform_mgr: &TransformMgr,
        model_mgr: &ModelMgr,
    ) -> bool {
        let transform_i = self.transform_i[index];
        let position = transform_mgr.position[transform_i];

        // OPTIMIZE: these are already calculated in calling function.
        let min_pos = self.get_translated_min_pos(index, position);
        let max_pos = self.get_translated_max_pos(index, position);

        let other_model_i = self.model_i[other_index].unwrap();
        let other_model = &model_mgr.model[other_model_i];

        let mut checks = 0;
        for mesh in (other_model.meshes).iter() {
            for vertex in (mesh.vertices).iter() {
                checks += 1;
                let x = vertex.position[0];
                let y = vertex.position[1];
                let z = vertex.position[2];
                if x <= max_pos.x
                    && x >= min_pos.x
                    && y <= max_pos.y
                    && y >= min_pos.y
                    && z <= max_pos.z
                    && z >= min_pos.z
                {
                    return true;
                }
            }
        }

        false
    }

    // fn check_collision_mesh_mesh(&self, index: usize, other_index: usize) -> bool {
    //     false
    // }

    /// Gets a translated minimum position, the [reference_position] argument is usually the transform's position.
    fn get_translated_min_pos(&self, index: usize, reference_position: Point3<f32>) -> Point3<f32> {
        let bounding_box_min = self.bounding_box_min[index];
        self.get_translated_position(reference_position, bounding_box_min)
    }

    /// Gets a translated maximum position, the [reference_position] argument is usually the transform's position.
    fn get_translated_max_pos(&self, index: usize, reference_position: Point3<f32>) -> Point3<f32> {
        let bounding_box_max = self.bounding_box_max[index];
        self.get_translated_position(reference_position, bounding_box_max)
    }

    fn get_translated_position(
        &self,
        reference_position: Point3<f32>,
        offset: Point3<f32>,
    ) -> Point3<f32> {
        let x = reference_position.x + offset.x;
        let y = reference_position.y + offset.y;
        let z = reference_position.z + offset.z;

        Point3 { x, y, z }
    }

    pub fn len(&self) -> usize {
        self.bounding_box_min.len()
    }

    pub async fn create_bounding_box_model(
        &self,
        bbox_min: &Point3<f32>,
        bbox_max: &Point3<f32>,
        render_state: &RenderState,
        mesh_renderer_mgr: &mut MeshInstancedRendererMgr,
    ) -> model::Model {
        let positions: [Vec<f32>; 8] = [
            vec![bbox_min[0], bbox_min[1], bbox_min[2]],
            vec![bbox_min[0], bbox_min[1], bbox_max[2]],
            vec![bbox_min[0], bbox_max[1], bbox_min[0]],
            vec![bbox_min[0], bbox_max[1], bbox_max[2]],
            vec![bbox_max[0], bbox_min[1], bbox_min[2]],
            vec![bbox_max[0], bbox_min[1], bbox_max[2]],
            vec![bbox_max[0], bbox_max[1], bbox_min[2]],
            vec![bbox_max[0], bbox_max[1], bbox_max[2]],
        ];

        let vertices = (0..positions.len())
            .map(|i| model::ModelVertex {
                position: [positions[i][0], positions[i][1], positions[i][2]],
                tex_coords: [0.0; 2],
                normal: [0.0; 3],
                tangent: [0.0; 3],
                bitangent: [0.0; 3],
            })
            .collect::<Vec<ModelVertex>>();

        let indices: Vec<u32> = vec![
            0, 1, 2, 1, 2, 3, 0, 4, 5, 0, 1, 5, 4, 5, 6, 5, 6, 7, 2, 6, 7, 2, 3, 7, 0, 2, 4, 0, 2,
            6, 1, 3, 7, 1, 3, 5,
        ];

        let vertex_buffer =
            render_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Collider bbox vertex buffer")),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        let index_buffer =
            render_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Collider bbox index buffer")),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

        let mesh = model::Mesh {
            name: String::from("Collider bounding box"),
            vertex_buffer,
            index_buffer,
            num_elements: indices.len() as u32,
            material: 0,

            vertices,
            is_wireframe: true,

            min_x: bbox_min[0],
            min_y: bbox_min[1],
            min_z: bbox_min[2],
            max_x: bbox_max[0],
            max_y: bbox_max[1],
            max_z: bbox_max[2],
        };

        model::Model::new_from_single_mesh(
            mesh,
            render_state,
            &mesh_renderer_mgr.texture_bind_group_layout,
        )
        .await
    }

    // pub fn render(
    //     &mut self,
    //     render_state: &RenderState,
    //     encoder: &wgpu::CommandEncoder,
    //     view: &wgpu::TextureView,
    // ) -> anyhow::Result<(), wgpu::SurfaceError> {
    //     let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    //         label: Some("Collider bounding box render pass"),
    //         color_attachments: &[Some(wgpu::RenderPassColorAttachment {
    //             view,
    //             resolve_target: None,
    //             ops: wgpu::Operations {
    //                 load: wgpu::LoadOp::Load,
    //                 store: wgpu::StoreOp::Store,
    //             },
    //         })],
    //         depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
    //             view: &render_state.depth_texture.view,
    //             depth_ops: Some(wgpu::Operations {
    //                 load: wgpu::LoadOp::Clear(1.0),
    //                 store: wgpu::StoreOp::Store,
    //             }),
    //             stencil_ops: None,
    //         }),
    //         timestamp_writes: None,
    //         occlusion_query_set: None,
    //     });

    //     render_pass.set_pipeline(&self.render_pipeline);
    //     render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    //     render_pass.set_bind_group(0, &render_state.camera_bind_group, &[]);
    //     render_pass.draw(0..AXIS_VERTICES.len() as u32, 0..1);

    //     Ok(())
    // }
}

pub enum ColliderType {
    /// Bounding box collision check.
    Box,
    /// Per-vertex collision check.
    Vertex,
}
