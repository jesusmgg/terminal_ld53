use std::{f32::consts::FRAC_PI_2, time::Duration};

use anyhow::Result;
use cgmath::{EuclideanSpace, InnerSpace, Point3, Quaternion, Rad, Vector3};

use crate::renderer::render_state::RenderState;

use super::{
    aircraft_input::AircraftInputMgr, collision::collider::ColliderMgr, inventory::InventoryMgr,
    mesh_renderer::MeshInstancedRendererMgr, model::ModelMgr, transform::TransformMgr,
};

const MAX_INSTANCE_COUNT: usize = 128;

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

    // TODO: research a safer way to store references
    pub inventory_i: Vec<Option<usize>>,

    pub transform_i: Vec<Option<usize>>,
    pub collider_i: Vec<Option<usize>>,
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

            inventory_i: Vec::with_capacity(MAX_INSTANCE_COUNT),

            transform_i: Vec::with_capacity(MAX_INSTANCE_COUNT),
            collider_i: Vec::with_capacity(MAX_INSTANCE_COUNT),
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

        inventory_mgr: &mut InventoryMgr,

        model_mgr: &mut ModelMgr,

        transform_mgr: &mut TransformMgr,
        collider_mgr: &mut ColliderMgr,
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

        let index = self.len() - 1;

        self.inventory_i.push(Some(inventory_mgr.add().unwrap()));

        self.transform_i
            .push(Some(transform_mgr.add(start_position, start_rotation)));
        self.input_i
            .push(Some(input_mgr.add(pilot_type.clone(), index)));

        let transform_i = self.transform_i[index].unwrap();
        let position = transform_mgr.position[transform_i];
        let rotation = transform_mgr.rotation[transform_i];

        let model_path = "models/Aircraft_1.obj";
        let model_i = model_mgr
            .get_with_name_or_add(model_path, &render_state, &mesh_renderer_mgr)
            .await;

        self.collider_i.push(Some(
            collider_mgr
                .add_from_model(model_i, transform_i, true, true, &model_mgr)
                .unwrap(),
        ));

        let mesh_renderer_i = match pilot_type {
            AircraftPilot::Player | AircraftPilot::Ai => {
                Some(mesh_renderer_mgr.add(render_state, model_i, position.to_vec(), rotation))
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

    pub fn update_player(
        &mut self,
        index: usize,
        transform_mgr: &mut TransformMgr,
        input_mgr: &mut AircraftInputMgr,
        dt: f32,
    ) {
        let i = index;
        let input_i = self.input_i[i].unwrap();
        let transform_i = self.transform_i[i].unwrap();

        // Get input
        let input_throttle = input_mgr.input_throttle[input_i];
        let input_reset_transform = input_mgr.input_reset_transform[input_i];
        let mut input_pitch = input_mgr.input_pitch[input_i];
        let mut input_yaw = input_mgr.input_yaw[input_i];

        // Throttle
        self.throttle[i] = Self::calculate_accumulated_speed(
            self.throttle[i],
            input_throttle,
            self.acceleration[i],
            self.min_speed[i],
            self.max_speed[i],
            dt,
        );

        // Update transform
        if input_reset_transform {
            Self::reset_transform(
                transform_i,
                self.start_position[i],
                self.start_rotation[i],
                transform_mgr,
            );
        } else {
            let translation = transform_mgr.forward(transform_i) * self.throttle[i] * dt;
            transform_mgr.translate(transform_i, translation);

            let forward = transform_mgr.forward(transform_i);
            let forward_y_cos = forward.dot(Vector3::unit_y());
            let right = transform_mgr.right(transform_i);
            let right_y_cos = right.dot(Vector3::unit_y());

            let current_pitch = forward_y_cos;
            let current_roll = right_y_cos;

            // Pitch
            // Input goes from -1 to 1. 0 means no input.
            // TODO: check performance of these comparisons (abs + float)
            if f32::abs(input_pitch) < 0.01 {
                if f32::abs(current_pitch) < 0.01 {
                    input_pitch = 0.0;
                    self.pitch_speed[i] = 0.0;
                } else {
                    input_pitch = -f32::signum(self.pitch_speed[i])
                }
            };
            self.pitch_speed[i] = Self::calculate_accumulated_speed(
                self.pitch_speed[i],
                input_pitch,
                self.pitch_acceleration[i],
                -self.pitch_max_speed[i],
                self.pitch_max_speed[i],
                dt,
            );

            // Yaw
            if f32::abs(input_yaw) < 0.01 {
                if f32::abs(self.yaw_speed[i]) < 0.01 {
                    input_yaw = 0.0;
                    self.yaw_speed[i] = 0.0;
                } else {
                    input_yaw = -f32::signum(self.yaw_speed[i]);
                }
            }
            self.yaw_speed[i] = Self::calculate_accumulated_speed(
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
            let roll_threshold: f32 = 0.2;
            let roll_delta = Rad(-current_roll
                + roll_threshold
                    * f32::sin(
                        FRAC_PI_2
                            * (self.yaw_speed[i] / self.yaw_max_speed[i])
                            * (1.0 - pitch_percent),
                    ));

            // Set rotations
            let mut flat_right = right;
            flat_right.y = 0.0;
            flat_right = flat_right.normalize();

            let mut flat_forward = forward;
            flat_forward.y = 0.0;
            flat_forward = flat_forward.normalize();

            transform_mgr.rotate_around_axis(transform_i, flat_right, pitch_delta);
            transform_mgr.rotate_around_axis(transform_i, Vector3::unit_y(), yaw_delta);
            transform_mgr.rotate_around_axis(transform_i, flat_forward, -roll_delta);
        }

        input_mgr.cleanup(input_i);
    }

    pub fn update_ai(&mut self, index: usize, transform_mgr: &mut TransformMgr, dt: f32) {
        let transform_i = self.transform_i[index].unwrap();
        let position_point = transform_mgr.position[transform_i];
        let position = position_point.to_vec();
        let rotation = transform_mgr.rotation[transform_i];
        let mut forward = transform_mgr.forward(transform_i);

        let player_index = self.get_player_aircraft_index();
        let player_transform_i = self.transform_i[player_index].unwrap();
        let player_position: Vector3<f32> = transform_mgr.position[player_transform_i].to_vec();
        let player_position_leveled =
            Vector3::new(player_position.x, position.y, player_position.z);

        // Horizontal rotation towards player direction
        let direction_leveled = player_position_leveled - position;
        let rotation_quat = Quaternion::from_arc(forward, direction_leveled.normalize(), None);
        let target_rotation = rotation * rotation_quat;

        let new_rotation = rotation
            .normalize()
            .slerp(target_rotation.normalize(), 3.0 * dt);

        transform_mgr.rotation[transform_i] = new_rotation;

        // Vertical translation
        let y_diff = player_position.y - position.y;
        forward.y = if f32::abs(y_diff) > 0.5 {
            f32::signum(y_diff)
        } else {
            0.0
        };

        // Throttle
        let speed = (direction_leveled.magnitude2() / 100.0)
            .clamp(self.min_speed[index], self.max_speed[index]);
        // let speed = 0.0;
        let translation = forward * speed * dt;
        transform_mgr.translate(transform_i, translation);
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

            match self.pilot_type[i] {
                AircraftPilot::Player => self.update_player(i, transform_mgr, input_mgr, dt),
                AircraftPilot::Ai => self.update_ai(i, transform_mgr, dt),
            }

            // Update mesh renderer
            // TODO: might be better off in render method
            if let Some(mesh_renderer_i) = self.mesh_renderer_i[i] {
                let position = transform_mgr.position[transform_i];
                let rotation = transform_mgr.rotation[transform_i];
                mesh_renderer_mgr.update_instance_position(
                    mesh_renderer_i,
                    position.to_vec(),
                    rotation,
                    render_state,
                );
            };
        }
    }

    fn calculate_accumulated_speed(
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

    pub fn ui(
        &self,
        transform_mgr: &TransformMgr,
        collider_mgr: &ColliderMgr,
        context: &egui::Context,
    ) {
        // Print player aircraft debug info
        let index = self.get_player_aircraft_index();
        let transform_i = self.transform_i[index].unwrap();
        let collider_i = self.collider_i[index].unwrap();

        let throttle = self.throttle[index];
        let position = transform_mgr.position[transform_i];
        let rotation = transform_mgr.rotation[transform_i];
        let forward = transform_mgr.forward(transform_i);
        let up = transform_mgr.up(transform_i);
        let right = transform_mgr.right(transform_i);

        let colliding_indices = collider_mgr.colliding_indices[collider_i];

        let throttle_str = format!("Throttle: {:?}", throttle);
        let position_str = format!("Position: {:?}", position);
        let rotation_str = format!("Rotation: {:?}", rotation);
        let forward_str = format!("Forward: {:?}", forward);
        let up_str = format!("Up: {:?}", up);
        let right_str = format!("Right: {:?}", right);
        let collisions_str = format!("Colliding indices: {:?}", colliding_indices);

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
                ui.label(collisions_str);
            });
    }

    fn reset_transform(
        transform_i: usize,
        start_position: Point3<f32>,
        start_rotation: Quaternion<f32>,
        transform_mgr: &mut TransformMgr,
    ) {
        transform_mgr.position[transform_i] = start_position;
        transform_mgr.rotation[transform_i] = start_rotation;
    }
}

#[derive(Clone)]
pub enum AircraftPilot {
    Player,
    Ai,
}
