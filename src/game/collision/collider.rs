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

    /// Updated each frame
    pub colliding_indices: Vec<[isize; MAX_COLLISIONS]>,

    // References
    transform_i: Vec<usize>,
}

impl ColliderMgr {
    pub fn new() -> ColliderMgr {
        ColliderMgr {
            bounding_box_min: Vec::with_capacity(MAX_INSTANCE_COUNT),
            bounding_box_max: Vec::with_capacity(MAX_INSTANCE_COUNT),

            colliding_indices: Vec::with_capacity(MAX_INSTANCE_COUNT),

            transform_i: Vec::with_capacity(MAX_INSTANCE_COUNT),
        }
    }

    pub fn add_from_model(
        &mut self,
        model_i: usize,
        transform_i: usize,
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

        self.colliding_indices.push([-1; MAX_COLLISIONS]);

        self.transform_i.push(transform_i);

        let index = self.len() - 1;

        Ok(index)
    }

    pub fn update(&mut self, transform_mgr: &TransformMgr) {
        // OPTIMIZE: parallelize collision checks
        for index in 0..self.len() {
            self.colliding_indices[index] = [-1; MAX_COLLISIONS];
        }
        for index in 0..self.len() {
            self.colliding_indices[index] = self.check_collisions(index, &transform_mgr)
        }
    }

    /// Checks and instance for collisions and returns colliding indexes.
    /// A -1 value in the returned array means no collision and should be ignored.
    fn check_collisions(
        &self,
        index: usize,
        transform_mgr: &TransformMgr,
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

            let other_transform_i = self.transform_i[other_index];
            let other_position = transform_mgr.position[other_transform_i];
            let other_min_pos = self.get_translated_min_pos(other_index, other_position);
            let other_max_pos = self.get_translated_max_pos(other_index, other_position);

            // Collision check
            if min_pos.x <= other_max_pos.x
                && max_pos.x >= other_min_pos.x
                && min_pos.y <= other_max_pos.y
                && max_pos.y >= other_min_pos.y
                && min_pos.z <= other_max_pos.z
                && max_pos.z >= other_min_pos.z
            {
                collisions[collisions_found] = other_index as isize;
                collisions_found += 1;
            }

            if collisions_found >= MAX_COLLISIONS {
                break;
            }
        }

        collisions
    }

    /// Gets a translated minimum position, the [reference_position] argument is usually the transform's position.
    fn get_translated_min_pos(&self, index: usize, reference_position: Point3<f32>) -> Point3<f32> {
        let bounding_box_min = self.bounding_box_min[index];
        let x = reference_position.x + bounding_box_min.x;
        let y = reference_position.y + bounding_box_min.y;
        let z = reference_position.z + bounding_box_min.z;

        Point3 { x, y, z }
    }

    /// Gets a translated maximum position, the [reference_position] argument is usually the transform's position.
    fn get_translated_max_pos(&self, index: usize, reference_position: Point3<f32>) -> Point3<f32> {
        let bounding_box_max = self.bounding_box_max[index];
        let x = reference_position.x + bounding_box_max.x;
        let y = reference_position.y + bounding_box_max.y;
        let z = reference_position.z + bounding_box_max.z;

        Point3 { x, y, z }
    }

    pub fn len(&self) -> usize {
        self.bounding_box_min.len()
    }
}
