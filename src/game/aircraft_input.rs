use cgmath::{InnerSpace, Vector3};
use winit::event::VirtualKeyCode;

use crate::input::keyboard_manager::KeyboardMgr;

use super::{
    aircraft::{AircraftMgr, AircraftPilot},
    transform::TransformMgr,
};

const MAX_INSTANCE_COUNT: usize = 128;

pub struct AircraftInputMgr {
    pub pilot_type: Vec<AircraftPilot>,

    pub input_yaw: Vec<f32>,
    pub input_pitch: Vec<f32>,
    pub input_throttle: Vec<f32>,

    pub input_reset_transform: Vec<bool>,

    pub aircraft_i: Vec<Option<usize>>,

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

        let ai_target_y_diff = self.rng.rand_range(1..20) as f32 + self.rng.rand_float();
        self.ai_target_y_diff.push(ai_target_y_diff);

        self.len() - 1
    }

    pub fn len(&self) -> usize {
        self.pilot_type.len()
    }

    pub fn update(
        &mut self,
        keyboard_mgr: &KeyboardMgr,
        aircraft_mgr: &AircraftMgr,
        transform_mgr: &TransformMgr,
    ) {
        for i in 0..self.pilot_type.len() {
            match self.pilot_type[i] {
                AircraftPilot::Player => {
                    self.process_keyboard_input(keyboard_mgr, i);
                }
                AircraftPilot::Ai => {
                    self.process_ai_input(i, aircraft_mgr, transform_mgr);
                }
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

    fn process_ai_input(
        &mut self,
        index: usize,
        aircraft_mgr: &AircraftMgr,
        transform_mgr: &TransformMgr,
    ) {
        // Get player data
        let player_aircraft_index = aircraft_mgr.get_player_aircraft_index();
        let player_transform_index = aircraft_mgr.transform_i[player_aircraft_index].unwrap();
        let player_position = transform_mgr.position[player_transform_index];

        // Get own data
        let aircraft_index = self.aircraft_i[index].unwrap();
        let transform_index = aircraft_mgr.transform_i[aircraft_index].unwrap();
        let position = transform_mgr.position[transform_index];
        let forward = transform_mgr.forward(transform_index);
        let current_pitch = forward.dot(Vector3::unit_y());
        let right = transform_mgr.right(transform_index);

        // Distances
        let distance = player_position - position;
        let distance_vector = distance.normalize();

        // Input pitch
        if f32::abs(distance.y) < self.ai_target_y_diff[index] {
            if f32::abs(current_pitch) > 0.01 {
                self.input_pitch[index] = f32::signum(-current_pitch);
            } else {
                self.input_pitch[index] = 0.0;
            }
        } else {
            self.input_pitch[index] = f32::signum(distance.y);
        }

        // Input yaw
        let right_distance_dot = right.dot(distance_vector);
        // Target is in front
        if forward.dot(distance_vector) > 0.0 {
            // Target is to the right
            if right_distance_dot > 0.0 {
                self.input_yaw[index] = 1.0;
            }
            // Target is to the left
            else {
                self.input_yaw[index] = -1.0;
            }
        }
        // Target is behind
        else {
            // Target is to the right
            if right_distance_dot > 0.0 {
                self.input_yaw[index] = -1.0;
            }
            // Target is to the left
            else {
                self.input_yaw[index] = 1.0;
            }
        }

        // Input throttle
        self.input_throttle[index] = 0.0;
    }

    pub fn cleanup(&mut self, index: usize) {
        self.input_yaw[index] = 0.0;
        self.input_pitch[index] = 0.0;
        self.input_throttle[index] = 0.0;
    }
}
