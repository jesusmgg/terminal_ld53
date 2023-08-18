use winit::event::VirtualKeyCode;

use crate::input::keyboard_manager::KeyboardMgr;

use super::aircraft::AircraftPilot;

const MAX_INSTANCE_COUNT: usize = 128;

pub struct AircraftInputMgr {
    pub pilot_type: Vec<AircraftPilot>,

    pub input_yaw: Vec<f32>,
    pub input_pitch: Vec<f32>,
    pub input_throttle: Vec<f32>,

    pub input_reset_transform: Vec<bool>,

    yaw_prev_dot: Vec<f32>,
    yaw_prev_sign: Vec<f32>,

    pub aircraft_i: Vec<Option<usize>>,

    // Height difference to maintain respect to target
    ai_target_y_diff: Vec<f32>,

    // TODO: make RNG app wide.
    rng: oorandom::Rand32,
}

impl AircraftInputMgr {
    pub fn new() -> Self {
        let rng = oorandom::Rand32::new(1234);

        Self {
            pilot_type: Vec::with_capacity(MAX_INSTANCE_COUNT),

            input_yaw: Vec::with_capacity(MAX_INSTANCE_COUNT),
            input_pitch: Vec::with_capacity(MAX_INSTANCE_COUNT),
            input_throttle: Vec::with_capacity(MAX_INSTANCE_COUNT),

            input_reset_transform: Vec::with_capacity(MAX_INSTANCE_COUNT),

            // Previous update yaw dot product.
            yaw_prev_dot: Vec::with_capacity(MAX_INSTANCE_COUNT),
            yaw_prev_sign: Vec::with_capacity(MAX_INSTANCE_COUNT),

            aircraft_i: Vec::with_capacity(MAX_INSTANCE_COUNT),

            ai_target_y_diff: Vec::with_capacity(MAX_INSTANCE_COUNT),

            rng,
        }
    }

    pub fn add(&mut self, pilot_type: AircraftPilot, aircraft_index: usize) -> usize {
        self.pilot_type.push(pilot_type);

        self.input_yaw.push(0.0);
        self.input_pitch.push(0.0);
        self.input_throttle.push(0.0);

        self.aircraft_i.push(Some(aircraft_index));

        self.input_reset_transform.push(false);

        self.yaw_prev_dot.push(-1.0);
        self.yaw_prev_sign.push(1.0);

        let ai_target_y_diff = self.rng.rand_range(1..20) as f32 + self.rng.rand_float();
        self.ai_target_y_diff.push(ai_target_y_diff);

        self.len() - 1
    }

    pub fn len(&self) -> usize {
        self.pilot_type.len()
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
        // TODO: use integers for input
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

        // Other
        self.input_reset_transform[index] = keyboard_mgr.key_down[VirtualKeyCode::R as usize];
    }

    pub fn cleanup(&mut self, index: usize) {
        self.input_yaw[index] = 0.0;
        self.input_pitch[index] = 0.0;
        self.input_throttle[index] = 0.0;
    }
}
