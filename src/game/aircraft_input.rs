use winit::event::VirtualKeyCode;

use crate::input::keyboard_manager::KeyboardMgr;

use super::aircraft::AircraftPilot;

const MAX_INSTANCE_COUNT: usize = 128;

pub struct AircraftInputMgr {
    pub pilot_type: Vec<AircraftPilot>,

    pub input_yaw: Vec<f32>,
    pub input_pitch: Vec<f32>,
    pub input_throttle: Vec<f32>,
}

impl AircraftInputMgr {
    pub fn new() -> Self {
        Self {
            pilot_type: Vec::with_capacity(MAX_INSTANCE_COUNT),

            input_yaw: Vec::with_capacity(MAX_INSTANCE_COUNT),
            input_pitch: Vec::with_capacity(MAX_INSTANCE_COUNT),
            input_throttle: Vec::with_capacity(MAX_INSTANCE_COUNT),
        }
    }

    pub fn add(&mut self, pilot_type: AircraftPilot) -> usize {
        self.pilot_type.push(pilot_type);

        self.input_yaw.push(0.0);
        self.input_pitch.push(0.0);
        self.input_throttle.push(0.0);

        self.pilot_type.len() - 1
    }

    pub fn update(&mut self, keyboard_mgr: &KeyboardMgr) {
        for i in 0..self.pilot_type.len() {
            match self.pilot_type[i] {
                AircraftPilot::Player => {
                    self.process_keyboard_input(keyboard_mgr, i);
                }
                AircraftPilot::Ai => {}
            }
        }
    }

    fn process_keyboard_input(&mut self, keyboard_mgr: &KeyboardMgr, index: usize) {
        let amount = 1.0;

        // Pitch
        if keyboard_mgr.key_pressed[VirtualKeyCode::Up as usize] {
            self.input_pitch[index] -= amount;
        }
        if keyboard_mgr.key_pressed[VirtualKeyCode::Down as usize] {
            self.input_pitch[index] += amount;
        }

        // Yaw
        if keyboard_mgr.key_pressed[VirtualKeyCode::Left as usize] {
            self.input_yaw[index] -= amount;
        }
        if keyboard_mgr.key_pressed[VirtualKeyCode::Right as usize] {
            self.input_yaw[index] += amount;
        }

        // Throttle
        if keyboard_mgr.key_pressed[VirtualKeyCode::A as usize] {
            self.input_throttle[index] += amount;
        }
        if keyboard_mgr.key_pressed[VirtualKeyCode::Z as usize] {
            self.input_throttle[index] -= amount;
        }
    }

    pub fn cleanup(&mut self, index: usize) {
        self.input_yaw[index] = 0.0;
        self.input_pitch[index] = 0.0;
    }
}
