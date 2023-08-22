use anyhow::Result;
use cgmath::Point3;

use crate::game::{model::ModelMgr, transform::TransformMgr};

const MAX_INSTANCE_COUNT: usize = 128;
/// Maximum simultaneous collisions per instance.
const MAX_COLLISIONS: usize = 4;

/// Manages bounding boxes and collisions.
pub struct ColliderMgr {
    bounding_box_min: Vec<Point3<f32>>,
    bounding_box_max: Vec<Point3<f32>>,

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

            transform_i: Vec::with_capacity(MAX_INSTANCE_COUNT),
            model_i: Vec::with_capacity(MAX_INSTANCE_COUNT),
        }
    }

    pub fn add_from_model(
        &mut self,
        model_i: usize,
        transform_i: usize,
        collider_type: ColliderType,
        is_collision_source: bool,
        is_collision_target: bool,
        model_mgr: &ModelMgr,
    ) -> Result<usize> {
        let model = &model_mgr.model[model_i];

        let bounding_box_min = Point3 {
            x: model.min_x,
            y: model.min_y,
            z: model.min_z,
        };
        let bounding_box_max = Point3 {
            x: model.max_x,
            y: model.max_y,
            z: model.max_z,
        };

        self.bounding_box_min.push(bounding_box_min);
        self.bounding_box_max.push(bounding_box_max);

        self.collider_type.push(collider_type);

        self.is_collision_source.push(is_collision_source);
        self.is_collision_target.push(is_collision_target);

        self.colliding_indices.push([-1; MAX_COLLISIONS]);

        self.transform_i.push(transform_i);
        self.model_i.push(Some(model_i));

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
}

pub enum ColliderType {
    /// Bounding box collision check.
    Box,
    /// Per-vertex collision check.
    Vertex,
}
