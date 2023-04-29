use std::time::Duration;

use cgmath::{Deg, InnerSpace, Rad, Vector3};
use winit::event::VirtualKeyCode;

use crate::{
    input::{cursor_manager::CursorMgr, keyboard_manager::KeyboardMgr},
    renderer::camera::Camera,
};

const SCROLL_BORDER_WIDTH: f32 = 50.0;
const SCROLL_BORDER_HEIGHT: f32 = 45.0;

pub struct RtsCameraController {
    speed: f32,
    initial_height: f32,
    min_height: f32,
    max_height: f32,
    angle: Rad<f32>,

    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,

    amount_up: f32,
    amount_down: f32,

    use_mouse: bool,
}

impl RtsCameraController {
    pub fn new(
        speed: f32,
        initial_height: f32,
        min_height: f32,
        max_height: f32,
        angle_deg: f32,
        camera: &mut Camera,
    ) -> RtsCameraController {
        let this = RtsCameraController {
            speed,
            initial_height,
            min_height,
            max_height,
            angle: Deg(angle_deg).into(),

            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,

            use_mouse: true,
        };

        this.reset_camera(camera);

        this
    }

    pub fn input(&mut self, keyboard_mgr: &KeyboardMgr) {
        if keyboard_mgr.key_down[VirtualKeyCode::M as usize] {
            self.use_mouse = !self.use_mouse;
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
        self.update_camera(camera, cursor_mgr, window, dt);
    }

    fn process_keyboard_input(&mut self, keyboard_mgr: &KeyboardMgr) {
        let amount = 1.0;

        if keyboard_mgr.key_pressed[VirtualKeyCode::W as usize]
            || keyboard_mgr.key_pressed[VirtualKeyCode::Up as usize]
        {
            self.amount_forward += amount;
        }
        if keyboard_mgr.key_pressed[VirtualKeyCode::S as usize]
            || keyboard_mgr.key_pressed[VirtualKeyCode::Down as usize]
        {
            self.amount_backward += amount;
        }
        if keyboard_mgr.key_pressed[VirtualKeyCode::A as usize]
            || keyboard_mgr.key_pressed[VirtualKeyCode::Left as usize]
        {
            self.amount_left += amount;
        }
        if keyboard_mgr.key_pressed[VirtualKeyCode::D as usize]
            || keyboard_mgr.key_pressed[VirtualKeyCode::Right as usize]
        {
            self.amount_right += amount;
        }
        if keyboard_mgr.key_pressed[VirtualKeyCode::Q as usize]
            || keyboard_mgr.key_pressed[VirtualKeyCode::LControl as usize]
        {
            self.amount_down += amount;
        }
        if keyboard_mgr.key_pressed[VirtualKeyCode::E as usize]
            || keyboard_mgr.key_pressed[VirtualKeyCode::Space as usize]
        {
            self.amount_up += amount;
        }
    }

    fn update_camera(
        &mut self,
        camera: &mut Camera,
        cursor_mgr: &CursorMgr,
        window: &winit::window::Window,
        dt: Duration,
    ) {
        let dt = dt.as_secs_f32();

        let mut position = camera.position;
        let pitch = self.angle;
        let yaw = camera.yaw;

        // Forward/backward and left/right with mouse
        if self.use_mouse {
            let window_size = window.inner_size();
            if cursor_mgr.x < SCROLL_BORDER_WIDTH {
                self.amount_left += 1.0;
            } else if cursor_mgr.x > window_size.width as f32 - SCROLL_BORDER_HEIGHT {
                self.amount_right += 1.0;
            }
            if cursor_mgr.y < SCROLL_BORDER_HEIGHT {
                self.amount_forward += 1.0;
            } else if cursor_mgr.y > window_size.height as f32 - SCROLL_BORDER_HEIGHT {
                self.amount_backward += 1.0;
            }
        }

        // Forward/backward and left/right with keyboard
        let (yaw_sin, yaw_cos) = yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        // Up/down
        position.y += (self.amount_up - self.amount_down) * self.speed * dt;
        if position.y > self.max_height {
            position.y = self.max_height;
        } else if position.y < self.min_height {
            position.y = self.min_height;
        }

        // Cleanup
        self.amount_forward = 0.0;
        self.amount_backward = 0.0;
        self.amount_right = 0.0;
        self.amount_left = 0.0;
        self.amount_up = 0.0;
        self.amount_down = 0.0;

        // Set camera
        camera.set(position, yaw, pitch, Rad(0.0));
    }

    /// Resets camera position, yaw and pitch to initial values
    fn reset_camera(&self, camera: &mut Camera) {
        let mut position = camera.position;
        let pitch = self.angle;
        let yaw = camera.yaw;

        position.y = self.initial_height;

        camera.set(position, yaw, pitch, Rad(0.0));
    }
}
