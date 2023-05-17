use cgmath::{
    perspective, InnerSpace, Matrix4, Point3, Quaternion, Rad, Rotation, SquareMatrix, Vector3,
};

use crate::game::transform::TransformMgr;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    pub rotation: Quaternion<f32>,
}

impl Camera {
    pub fn new(position: Point3<f32>, rotation: Quaternion<f32>) -> Self {
        Self { position, rotation }
    }

    pub fn set_from_transform_mgr(&mut self, transform_mgr: &TransformMgr, index: usize) {
        self.position = transform_mgr.position[index];
        self.rotation = transform_mgr.rotation[index];
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let forward = self.rotation.rotate_vector(Vector3::unit_z()).normalize();
        let up = self.rotation.rotate_vector(Vector3::unit_y()).normalize();

        Matrix4::look_to_rh(self.position, forward, up)
    }
}

pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
        let view_matrix = camera.calc_matrix();

        self.view_position = camera.position.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * view_matrix).into();
    }
}
