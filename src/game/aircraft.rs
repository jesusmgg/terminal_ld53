use std::time::Duration;

use anyhow::Result;
use cgmath::{Deg, EuclideanSpace, InnerSpace, Point3, Quaternion, Rad, Rotation3, Vector3};

use crate::{renderer::render_state::RenderState, resources};

use super::{
    aircraft_input::AircraftInputMgr, mesh_renderer::MeshInstancedRendererMgr,
    transform::TransformMgr,
};

const MAX_INSTANCE_COUNT: usize = 128;

const HALF_PI: f32 = 1.57079632679;

/// Represents aircraft, both player and enemy.
/// Aircraft index 0 is always the player.
pub struct AircraftMgr {
    pilot_type: Vec<AircraftPilot>,

    throttle: Vec<f32>,
    max_speed: Vec<f32>,
    min_speed: Vec<f32>,
    acceleration: Vec<f32>,

    yaw_speed: Vec<f32>,
    yaw_max_speed: Vec<f32>,
    yaw_acceleration: Vec<f32>,

    pitch_speed: Vec<f32>,
    pitch_max_speed: Vec<f32>,
    pitch_acceleration: Vec<f32>,

    start_position: Vec<Point3<f32>>,
    start_rotation: Vec<Quaternion<f32>>,

    // TODO: choose a safer way to store references
    pub transform_i: Vec<Option<usize>>,
    pub input_i: Vec<Option<usize>>,
    pub mesh_renderer_i: Vec<Option<usize>>,
}

impl AircraftMgr {
    pub fn new() -> Result<AircraftMgr> {
        Ok(AircraftMgr {
            pilot_type: Vec::with_capacity(MAX_INSTANCE_COUNT),

            throttle: Vec::with_capacity(MAX_INSTANCE_COUNT),
            max_speed: Vec::with_capacity(MAX_INSTANCE_COUNT),
            min_speed: Vec::with_capacity(MAX_INSTANCE_COUNT),
            acceleration: Vec::with_capacity(MAX_INSTANCE_COUNT),

            yaw_speed: Vec::with_capacity(MAX_INSTANCE_COUNT),
            yaw_max_speed: Vec::with_capacity(MAX_INSTANCE_COUNT),
            yaw_acceleration: Vec::with_capacity(MAX_INSTANCE_COUNT),

            pitch_speed: Vec::with_capacity(MAX_INSTANCE_COUNT),
            pitch_max_speed: Vec::with_capacity(MAX_INSTANCE_COUNT),
            pitch_acceleration: Vec::with_capacity(MAX_INSTANCE_COUNT),

            start_position: Vec::with_capacity(MAX_INSTANCE_COUNT),
            start_rotation: Vec::with_capacity(MAX_INSTANCE_COUNT),

            transform_i: Vec::with_capacity(MAX_INSTANCE_COUNT),
            input_i: Vec::with_capacity(MAX_INSTANCE_COUNT),
            mesh_renderer_i: Vec::with_capacity(MAX_INSTANCE_COUNT),
        })
    }

    pub async fn add(
        &mut self,

        pilot_type: AircraftPilot,

        max_speed: f32,
        min_speed: f32,
        acceleration: f32,

        yaw_max_speed: f32,
        yaw_acceleration: f32,

        pitch_max_speed: f32,
        pitch_acceleration: f32,

        start_position: Point3<f32>,
        start_rotation: Quaternion<f32>,

        transform_mgr: &mut TransformMgr,
        input_mgr: &mut AircraftInputMgr,
        mesh_renderer_mgr: &mut MeshInstancedRendererMgr,

        render_state: &RenderState,
    ) -> Result<usize> {
        self.pilot_type.push(pilot_type.clone());

        self.throttle.push(min_speed);
        self.max_speed.push(max_speed);
        self.min_speed.push(min_speed);
        self.acceleration.push(acceleration);

        self.yaw_speed.push(0.0);
        self.yaw_max_speed.push(yaw_max_speed);
        self.yaw_acceleration.push(yaw_acceleration);

        self.pitch_speed.push(0.0);
        self.pitch_max_speed.push(pitch_max_speed);
        self.pitch_acceleration.push(pitch_acceleration);

        self.start_position.push(start_position);
        self.start_rotation.push(start_rotation);

        self.transform_i
            .push(Some(transform_mgr.add(start_position, start_rotation)));
        self.input_i.push(Some(input_mgr.add(pilot_type.clone())));

        let index = self.len() - 1;

        let transform_i = self.transform_i[index].unwrap();
        let position = transform_mgr.position[transform_i];
        let rotation = transform_mgr.rotation[transform_i];

        let mesh_renderer_i = match pilot_type {
            AircraftPilot::Player => None,

            AircraftPilot::Ai => {
                // TODO: load model a single time and not for each aircraft instance.
                let aircraft_1_model = resources::load_model_obj(
                    "models/Aircraft_1.obj",
                    &render_state.device,
                    &render_state.queue,
                    &mesh_renderer_mgr.texture_bind_group_layout,
                )
                .await
                .unwrap();

                Some(mesh_renderer_mgr.add(
                    render_state,
                    aircraft_1_model,
                    position.clone().to_vec(),
                    rotation,
                ))
            }
        };
        self.mesh_renderer_i.push(mesh_renderer_i);

        Ok(index)
    }

    pub fn get_player_aircraft_index(&self) -> usize {
        0
    }

    pub fn len(&self) -> usize {
        self.pilot_type.len()
    }

    pub fn update(
        &mut self,
        transform_mgr: &mut TransformMgr,
        input_mgr: &mut AircraftInputMgr,
        mesh_renderer_mgr: &mut MeshInstancedRendererMgr,
        render_state: &RenderState,
        dt: Duration,
    ) {
        let dt = dt.as_secs_f32();

        for i in 0..self.len() {
            let transform_i = self.transform_i[i].unwrap();
            let input_i = self.input_i[i].unwrap();

            // Throttle
            self.throttle[i] = self.calculate_accumulated_speed(
                self.throttle[i],
                input_mgr.input_throttle[input_i],
                self.acceleration[i],
                self.min_speed[i],
                self.max_speed[i],
                dt,
            );

            // Update transform
            if input_mgr.input_reset_transform[input_i] {
                self.reset_transform(i, transform_mgr);
            } else {
                let translation = transform_mgr.forward(transform_i) * self.throttle[i] * dt;
                transform_mgr.translate(transform_i, translation);

                let right = transform_mgr.right(transform_i);
                let right_y_cos = right.dot(Vector3::unit_y());
                let forward = transform_mgr.forward(transform_i);
                let forward_y_cos = forward.dot(Vector3::unit_y());

                // Pitch
                // Input goes from -1 to 1. 0 means no input.
                let current_pitch = forward_y_cos;
                let mut input_pitch = input_mgr.input_pitch[input_i];
                // TODO: check performance of these comparisons (abs + float)
                if f32::abs(input_pitch) < 0.01 {
                    if f32::abs(current_pitch) < 0.01 {
                        input_pitch = 0.0;
                        self.pitch_speed[i] = 0.0;
                    } else {
                        input_pitch = -f32::signum(self.pitch_speed[i])
                    }
                };
                self.pitch_speed[i] = self.calculate_accumulated_speed(
                    self.pitch_speed[i],
                    input_pitch,
                    self.pitch_acceleration[i],
                    -self.pitch_max_speed[i],
                    self.pitch_max_speed[i],
                    dt,
                );

                // Yaw
                let mut input_yaw = input_mgr.input_yaw[input_i];
                if f32::abs(input_yaw) < 0.01 {
                    if f32::abs(self.yaw_speed[i]) < 0.01 {
                        input_yaw = 0.0;
                        self.yaw_speed[i] = 0.0;
                    } else {
                        input_yaw = -f32::signum(self.yaw_speed[i]);
                    }
                }
                self.yaw_speed[i] = self.calculate_accumulated_speed(
                    self.yaw_speed[i],
                    input_yaw,
                    self.yaw_acceleration[i],
                    -self.yaw_max_speed[i],
                    self.yaw_max_speed[i],
                    dt,
                );

                let mut pitch_delta = Rad(self.pitch_speed[i] * dt);
                let yaw_delta = Rad(self.yaw_speed[i] * dt);

                // Limit pitch
                let pitch_threshold: f32 = 0.9; // ~84.26 degrees
                let pitch_percent = f32::abs(current_pitch / pitch_threshold);
                if (current_pitch < -pitch_threshold && pitch_delta < Rad(0.0))
                    || (current_pitch > pitch_threshold && pitch_delta > Rad(0.0))
                {
                    pitch_delta = Rad(0.0);
                    self.pitch_speed[i] = 0.0;
                }

                // Roll
                let roll_threshold: f32 = 0.5;
                let current_roll = right_y_cos;
                let roll_delta = Rad(-current_roll
                    + roll_threshold
                        * f32::sin(
                            HALF_PI
                                * (self.yaw_speed[i] / self.yaw_max_speed[i])
                                * (1.0 - pitch_percent),
                        ));

                // Set rotations
                let mut flat_right = right;
                flat_right.y = 0.0;
                flat_right.normalize();

                let mut flat_forward = forward;
                flat_forward.y = 0.0;
                flat_forward.normalize();

                transform_mgr.rotate_around_axis(transform_i, flat_right, pitch_delta);
                transform_mgr.rotate_around_axis(transform_i, Vector3::unit_y(), yaw_delta);
                transform_mgr.rotate_around_axis(transform_i, flat_forward, -roll_delta);
            }

            input_mgr.cleanup(input_i);

            // Update mesh renderer
            // TODO: might be better off in render method
            match self.mesh_renderer_i[i] {
                Some(mesh_renderer_i) => {
                    let position = transform_mgr.position[transform_i];
                    let rotation = Quaternion::from_axis_angle(Vector3::unit_z(), Deg(0.0));
                    mesh_renderer_mgr.update_instance_position(
                        mesh_renderer_i,
                        position.to_vec(),
                        rotation,
                        &render_state,
                    );
                }
                None => {}
            };
        }
    }

    fn calculate_accumulated_speed(
        &self,
        current_speed: f32,
        input: f32,
        acceleration: f32,
        min_speed: f32,
        max_speed: f32,
        dt: f32,
    ) -> f32 {
        let mut speed = current_speed;
        speed += input * acceleration * dt;

        if speed > max_speed {
            return max_speed;
        } else if speed < min_speed {
            return min_speed;
        }

        if f32::abs(speed) < 0.001 {
            return 0.0;
        }

        speed
    }

    pub fn ui(&self, transform_mgr: &TransformMgr, context: &egui::Context) {
        // Print player aircraft debug info
        let index = self.get_player_aircraft_index();
        let throttle = self.throttle[index];
        let position = transform_mgr.position[index];
        let rotation = transform_mgr.rotation[index];
        let forward = transform_mgr.forward(index);
        let up = transform_mgr.up(index);
        let right = transform_mgr.right(index);

        let throttle_str = format!("Throttle: {:?}", throttle);
        let position_str = format!("Position: {:?}", position);
        let rotation_str = format!("Rotation: {:?}", rotation);
        let forward_str = format!("Forward: {:?}", forward);
        let up_str = format!("Up: {:?}", up);
        let right_str = format!("Right: {:?}", right);

        egui::SidePanel::left("Player Aircraft")
            .resizable(false)
            .min_width(400.0)
            .show(context, |ui| {
                ui.label("Player aircraft");
                ui.label("----------------------");
                ui.label(throttle_str);
                ui.label(position_str);
                ui.label(rotation_str);
                ui.label(forward_str);
                ui.label(up_str);
                ui.label(right_str);
            });
    }

    fn reset_transform(&self, index: usize, transform_mgr: &mut TransformMgr) {
        let transform_i = self.transform_i[index];

        match transform_i {
            Some(transform_i) => {
                transform_mgr.position[transform_i] = self.start_position[index];
                transform_mgr.rotation[transform_i] = self.start_rotation[index];
            }
            None => {}
        }
    }
}

#[derive(Clone)]
pub enum AircraftPilot {
    Player,
    Ai,
}
