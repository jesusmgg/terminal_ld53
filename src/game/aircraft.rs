use std::time::Duration;

use anyhow::Result;
use cgmath::{Deg, EuclideanSpace, Point3, Quaternion, Rad, Rotation3, Vector3};

use crate::{renderer::render_state::RenderState, resources};

use super::{
    aircraft_input::AircraftInputMgr, mesh_renderer::MeshInstancedRendererMgr,
    transform::TransformMgr,
};

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
            yaw_speed: Vec::with_capacity(MAX_INSTANCE_COUNT),
            pitch_speed: Vec::with_capacity(MAX_INSTANCE_COUNT),

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
        yaw_speed: f32,
        pitch_speed: f32,

        start_position: Point3<f32>,

        transform_mgr: &mut TransformMgr,
        input_mgr: &mut AircraftInputMgr,
        mesh_renderer_mgr: &mut MeshInstancedRendererMgr,

        render_state: &RenderState,
    ) -> Result<usize> {
        self.pilot_type.push(pilot_type.clone());

        self.throttle.push(min_speed);
        self.max_speed.push(max_speed);
        self.min_speed.push(min_speed);
        self.yaw_speed.push(yaw_speed);
        self.pitch_speed.push(pitch_speed);

        self.transform_i
            .push(Some(transform_mgr.add(start_position)));
        self.input_i.push(Some(input_mgr.add(pilot_type.clone())));

        let index = self.len() - 1;

        let position = transform_mgr.position[self.transform_i[index].unwrap()];
        let rotation = Quaternion::from_axis_angle(Vector3::unit_z(), Deg(0.0));

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
            if self.throttle[i] > self.max_speed[i] {
                self.throttle[i] = self.max_speed[i]
            } else if self.throttle[i] < self.min_speed[i] {
                self.throttle[i] = self.min_speed[i];
            }

            // Update values
            let translation = transform_mgr.get_forward(transform_i) * self.throttle[i] * dt;
            transform_mgr.translate(transform_i, translation);

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
}

#[derive(Clone)]
pub enum AircraftPilot {
    Player,
    Ai,
}
