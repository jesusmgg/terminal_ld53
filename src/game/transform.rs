use cgmath::{Euler, InnerSpace, Point3, Quaternion, Rad, Rotation, Rotation3, Vector3};
const MAX_INSTANCE_COUNT: usize = 128;

// OPTIMIZE: Rotation calculations are allocating new Quaternions and Eulers all the time.
//           An approach could be caching up, right, forward, pitch, yaw, roll values and keeping the dirty state.
pub struct TransformMgr {
    pub position: Vec<Point3<f32>>,
    pub rotation: Vec<Quaternion<f32>>,
}

impl TransformMgr {
    pub fn new() -> TransformMgr {
        TransformMgr {
            position: Vec::with_capacity(MAX_INSTANCE_COUNT),
            rotation: Vec::with_capacity(MAX_INSTANCE_COUNT),
        }
    }

    /// Returns instance index
    pub fn add<V: Into<Point3<f32>>>(&mut self, position: V, rotation: Quaternion<f32>) -> usize {
        self.position.push(position.into());
        self.rotation.push(rotation);

        self.len() - 1
    }

    /// Returns the amount of managed instances.
    pub fn len(&self) -> usize {
        self.position.len()
    }

    pub fn update(&mut self) {}

    /// Rotate local principal rotation axes.
    /// Takes pitch, yaw and roll as Rad<f32>.
    pub fn rotate_local_axes(
        &mut self,
        index: usize,
        pitch_delta: Rad<f32>,
        yaw_delta: Rad<f32>,
        roll_delta: Rad<f32>,
    ) {
        let z_rotation = Quaternion::from_axis_angle(self.forward(index), -roll_delta);
        let y_rotation = Quaternion::from_axis_angle(self.up(index), -yaw_delta);
        let x_rotation = Quaternion::from_axis_angle(self.right(index), -pitch_delta);

        let combined_rotation = x_rotation * y_rotation * z_rotation;

        self.rotation[index] = combined_rotation * self.rotation[index];
    }

    pub fn rotate_around_axis(&mut self, index: usize, axis: Vector3<f32>, angle: Rad<f32>) {
        let rotation = Quaternion::from_axis_angle(axis, -angle);
        self.rotation[index] = rotation * self.rotation[index];
    }

    pub fn euler(&self, index: usize) -> Euler<Rad<f32>> {
        Euler::from(self.rotation[index])
    }

    pub fn forward(&self, index: usize) -> Vector3<f32> {
        self.rotation[index]
            .rotate_vector(Vector3::unit_z())
            .normalize()
    }

    pub fn right(&self, index: usize) -> Vector3<f32> {
        self.rotation[index]
            .rotate_vector(Vector3::unit_x())
            .normalize()
    }

    pub fn up(&self, index: usize) -> Vector3<f32> {
        self.rotation[index]
            .rotate_vector(Vector3::unit_y())
            .normalize()
    }

    pub fn translate(&mut self, index: usize, translation: Vector3<f32>) {
        self.position[index] += translation;
    }
}
