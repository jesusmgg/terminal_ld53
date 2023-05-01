use std::f32::consts::FRAC_PI_2;

use cgmath::{InnerSpace, Point3, Quaternion, Rad, Rotation3, Vector3};

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

const MAX_INSTANCE_COUNT: usize = 128;

// TODO: add rotation
pub struct TransformMgr {
    pub position: Vec<Point3<f32>>,

    forward: Vec<Vector3<f32>>,
    right: Vec<Vector3<f32>>,
    up: Vec<Vector3<f32>>,

    pub pitch: Vec<Rad<f32>>,
    pub yaw: Vec<Rad<f32>>,
    pub roll: Vec<Rad<f32>>,

    is_dirty: Vec<bool>,
}

impl TransformMgr {
    pub fn new() -> TransformMgr {
        TransformMgr {
            position: Vec::with_capacity(MAX_INSTANCE_COUNT),

            forward: Vec::with_capacity(MAX_INSTANCE_COUNT),
            right: Vec::with_capacity(MAX_INSTANCE_COUNT),
            up: Vec::with_capacity(MAX_INSTANCE_COUNT),

            pitch: Vec::with_capacity(MAX_INSTANCE_COUNT),
            yaw: Vec::with_capacity(MAX_INSTANCE_COUNT),
            roll: Vec::with_capacity(MAX_INSTANCE_COUNT),

            is_dirty: Vec::with_capacity(MAX_INSTANCE_COUNT),
        }
    }

    /// Returns instance index
    pub fn add<V: Into<Point3<f32>>>(&mut self, position: V) -> usize {
        self.position.push(position.into());

        self.forward.push(-Vector3::unit_z());
        self.right.push(Vector3::unit_x());
        self.up.push(Vector3::unit_y());

        self.pitch.push(Rad(0.0));
        self.yaw.push(Rad(0.0));
        self.roll.push(Rad(0.0));

        self.is_dirty.push(true);

        self.position.len() - 1
    }

    pub fn update(&mut self) {
        for i in 0..self.position.len() {
            if self.is_dirty[i] {
                let (pitch_sin, pitch_cos) = self.pitch[i].0.sin_cos();
                let (yaw_sin, yaw_cos) = self.yaw[i].0.sin_cos();

                let forward =
                    Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
                let mut right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
                right = Quaternion::from_axis_angle(forward, self.roll[i]) * right;
                let up = Vector3::cross(right, forward).normalize();

                self.forward[i] = forward;
                self.right[i] = right;
                self.up[i] = up;

                self.is_dirty[i] = false;
            }
        }
    }

    pub fn set_pitch(&mut self, index: usize, mut pitch: Rad<f32>) {
        // Limit angle
        if pitch < -Rad(SAFE_FRAC_PI_2) {
            pitch = -Rad(SAFE_FRAC_PI_2);
        } else if pitch > Rad(SAFE_FRAC_PI_2) {
            pitch = Rad(SAFE_FRAC_PI_2);
        }

        self.pitch[index] = pitch;
        self.is_dirty[index] = true
    }

    pub fn set_yaw(&mut self, index: usize, yaw: Rad<f32>) {
        self.yaw[index] = yaw;
        self.is_dirty[index] = true
    }
    pub fn set_roll(&mut self, index: usize, roll: Rad<f32>) {
        self.roll[index] = roll;
        self.is_dirty[index] = true
    }

    pub fn get_forward(&self, index: usize) -> Vector3<f32> {
        self.forward[index]
    }
    pub fn get_right(&self, index: usize) -> Vector3<f32> {
        self.right[index]
    }
    pub fn get_up(&self, index: usize) -> Vector3<f32> {
        self.up[index]
    }

    pub fn translate(&mut self, index: usize, translation: Vector3<f32>) {
        self.position[index] += translation;
    }
}
