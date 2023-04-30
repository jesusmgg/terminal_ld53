use std::time::Duration;

use cgmath::{Point3, Rad};

use super::{aircraft_input::AircraftInputMgr, transform::TransformMgr};

const MAX_INSTANCE_COUNT: usize = 128;

/// Represents aircraft, both player and enemy.
/// Aircraft index 0 is always the player.
pub struct AircraftMgr {
    pilot_type: Vec<AircraftPilot>,

    throttle: Vec<f32>,
    max_speed: Vec<f32>,
    min_speed: Vec<f32>,
    yaw_speed: Vec<f32>,
    pitch_speed: Vec<f32>,

    pub transform_i: Vec<usize>,
    pub input_i: Vec<usize>,
}

impl AircraftMgr {
    pub fn new() -> AircraftMgr {
        AircraftMgr {
            pilot_type: Vec::with_capacity(MAX_INSTANCE_COUNT),

            throttle: Vec::with_capacity(MAX_INSTANCE_COUNT),
            max_speed: Vec::with_capacity(MAX_INSTANCE_COUNT),
            min_speed: Vec::with_capacity(MAX_INSTANCE_COUNT),
            yaw_speed: Vec::with_capacity(MAX_INSTANCE_COUNT),
            pitch_speed: Vec::with_capacity(MAX_INSTANCE_COUNT),

            transform_i: Vec::with_capacity(MAX_INSTANCE_COUNT),
            input_i: Vec::with_capacity(MAX_INSTANCE_COUNT),
        }
    }

    pub fn add(
        &mut self,

        pilot_type: AircraftPilot,

        max_speed: f32,
        min_speed: f32,
        yaw_speed: f32,
        pitch_speed: f32,

        start_position: Point3<f32>,

        transform_mgr: &mut TransformMgr,
        input_mgr: &mut AircraftInputMgr,
    ) -> usize {
        self.pilot_type.push(pilot_type.clone());

        self.throttle.push(min_speed);
        self.max_speed.push(max_speed);
        self.min_speed.push(min_speed);
        self.yaw_speed.push(yaw_speed);
        self.pitch_speed.push(pitch_speed);

        self.transform_i.push(transform_mgr.add(start_position));
        self.input_i.push(input_mgr.add(pilot_type.clone()));

        self.pilot_type.len() - 1
    }

    pub fn get_player_aircraft_index(&self) -> usize {
        0
    }

    pub fn update(
        &mut self,
        transform_mgr: &mut TransformMgr,
        input_mgr: &mut AircraftInputMgr,
        dt: Duration,
    ) {
        let dt = dt.as_secs_f32();

        for i in 0..self.throttle.len() {
            let transform_i = self.transform_i[i];
            let input_i = self.input_i[i];

            // Throttle
            if self.throttle[i] > self.max_speed[i] {
                self.throttle[i] = self.max_speed[i]
            } else if self.throttle[i] < self.min_speed[i] {
                self.throttle[i] = self.min_speed[i];
            }

            // Update values
            transform_mgr.position[transform_i] +=
                transform_mgr.forward[transform_i] * self.throttle[i] * dt;

            transform_mgr.set_yaw(
                transform_i,
                transform_mgr.yaw[transform_i]
                    + Rad(input_mgr.input_yaw[input_i]) * self.yaw_speed[i] * dt,
            );
            transform_mgr.set_pitch(
                transform_i,
                transform_mgr.pitch[transform_i]
                    + Rad(input_mgr.input_pitch[input_i]) * self.pitch_speed[i] * dt,
            );
            // roll += self.input_roll * 0.2 * dt;

            input_mgr.cleanup(input_i);
        }
    }
}

#[derive(Clone)]
pub enum AircraftPilot {
    Player,
    Ai,
}
