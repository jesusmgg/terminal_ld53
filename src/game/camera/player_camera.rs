use std::time::Duration;

use cgmath::{InnerSpace, Point3, Rad, Vector3};
use winit::event::VirtualKeyCode;

use std::f32::consts::FRAC_PI_2;

use crate::{
    input::{cursor_manager::CursorMgr, keyboard_manager::KeyboardMgr},
    renderer::camera::Camera,
};

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

pub struct PlayerCameraController {
    input_left: f32,
    input_right: f32,
    input_forward: f32,
    input_up: f32,
    input_down: f32,

    sensitivity: f32,

    speed: f32,

    pub position: Point3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
    // TODO: support roll
}

impl PlayerCameraController {
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
        speed: f32,
        sensitivity: f32,
    ) -> PlayerCameraController {
        PlayerCameraController {
            input_left: 0.0,
            input_right: 0.0,
            input_forward: 0.0,
            input_up: 0.0,
            input_down: 0.0,

            sensitivity,

            speed,

            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }

    pub fn update(
        &mut self,
        camera: &mut Camera,
        cursor_mgr: &CursorMgr,
        keyboard_mgr: &KeyboardMgr,
        window: &winit::window::Window,
        dt: Duration,
    ) {
        self.process_keyboard_input(keyboard_mgr);
        self.update_camera(camera, dt);
    }

    fn process_keyboard_input(&mut self, keyboard_mgr: &KeyboardMgr) {
        let amount = 1.0;

        if keyboard_mgr.key_pressed[VirtualKeyCode::Up as usize] {
            self.input_down += amount;
        }
        if keyboard_mgr.key_pressed[VirtualKeyCode::Down as usize] {
            self.input_up += amount;
        }
        if keyboard_mgr.key_pressed[VirtualKeyCode::Left as usize] {
            self.input_left += amount;
        }
        if keyboard_mgr.key_pressed[VirtualKeyCode::Right as usize] {
            self.input_right += amount;
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        self.input_forward = 1.0;

        let mut position = camera.position;
        let mut pitch = camera.pitch;
        let mut yaw = camera.yaw;

        let (yaw_sin, yaw_cos) = yaw.0.sin_cos();
        let pitch_sin = pitch.0.sin();
        let forward = Vector3::new(yaw_cos, pitch_sin, yaw_sin).normalize();
        position += forward * self.input_forward * self.speed * dt;

        // Up/down
        position.y += (self.input_up - self.input_down) * self.speed * dt;

        // Rotate
        yaw += Rad(self.input_right - self.input_left) * self.sensitivity * dt;
        pitch += Rad(self.input_up - self.input_down) * self.sensitivity * dt;

        // Limit camera angle
        if pitch < -Rad(SAFE_FRAC_PI_2) {
            pitch = -Rad(SAFE_FRAC_PI_2);
        } else if pitch > Rad(SAFE_FRAC_PI_2) {
            pitch = Rad(SAFE_FRAC_PI_2);
        }

        // Cleanup
        self.input_forward = 0.0;
        self.input_right = 0.0;
        self.input_left = 0.0;
        self.input_up = 0.0;
        self.input_down = 0.0;

        // Set camera
        camera.set(position, yaw, pitch);
    }

    // Resets camera position, yaw and pitch to initial values
    // fn reset_camera(&self, camera: &mut Camera) {
    //     let mut position = camera.position;
    //     let pitch = self.angle;
    //     let yaw = camera.yaw;

    //     position.y = self.initial_height;

    //     camera.set(position, yaw, pitch);
    // }
}
