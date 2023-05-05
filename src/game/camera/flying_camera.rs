use std::time::Duration;

use winit::event::{
    DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode,
    WindowEvent,
};

use std::f32::consts::FRAC_PI_2;

use crate::renderer::camera::Camera;

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

pub struct FlyingCameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
    mouse_pressed: bool,
}

impl FlyingCameraController {
    pub fn new(speed: f32, sensitivity: f32) -> FlyingCameraController {
        FlyingCameraController {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
            mouse_pressed: false,
        }
    }

    pub fn input<T>(&mut self, event: &winit::event::Event<T>, window: &winit::window::Window) {
        match *event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                self.process_mouse(delta.0, delta.1);
            }

            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(key),
                            state,
                            ..
                        },
                    ..
                } => self.process_keyboard(*key, *state),
                WindowEvent::MouseWheel { delta, .. } => {
                    self.process_scroll(delta);
                }
                WindowEvent::MouseInput {
                    button: MouseButton::Left,
                    state,
                    ..
                } => {
                    // TODO: save mouse pressed status
                    self.mouse_pressed = *state == ElementState::Pressed;
                }
                _ => (),
            },
            _ => (),
        }
    }

    pub fn update(&mut self, camera: &mut Camera, dt: Duration) {
        self.update_camera(camera, dt);
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) {
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };
        match key {
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                self.amount_forward = amount;
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                self.amount_backward = amount;
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                self.amount_left = amount;
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                self.amount_right = amount;
            }
            VirtualKeyCode::Q | VirtualKeyCode::LControl => {
                self.amount_down = amount;
            }
            VirtualKeyCode::E | VirtualKeyCode::Space => {
                self.amount_up = amount;
            }
            _ => (),
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = match delta {
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0, // line ~ 100px
            MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition { y: scroll, .. }) => {
                *scroll as f32
            }
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        // TODO: update camera using transform manager
        /*
        if !self.mouse_pressed {
            return;
        }

        let dt = dt.as_secs_f32();

        let mut position = camera.position;
        let mut pitch = camera.pitch;
        let mut yaw = camera.yaw;

        // Forward/backward and left/right
        let (yaw_sin, yaw_cos) = yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        // In/out (sort of zoom, but camera actually moves)
        let (pitch_sin, pitch_cos) = pitch.0.sin_cos();
        let scrollward = Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin);
        position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        // Up/down
        position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        // Rotate
        yaw += Rad(self.rotate_horizontal) * self.sensitivity * dt;
        pitch += Rad(-self.rotate_vertical) * self.sensitivity * dt;

        // Reset rotation
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Limit camera angle
        if pitch < -Rad(SAFE_FRAC_PI_2) {
            pitch = -Rad(SAFE_FRAC_PI_2);
        } else if pitch > Rad(SAFE_FRAC_PI_2) {
            pitch = Rad(SAFE_FRAC_PI_2);
        }

        camera.set_position_pitch_yaw_roll(position, pitch, yaw, Rad(0.0));
        */
    }
}
