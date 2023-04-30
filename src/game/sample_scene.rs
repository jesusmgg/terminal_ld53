use cgmath::Rotation3;

use crate::{renderer::render_state::RenderState, resources};

use super::{
    aircraft::{AircraftMgr, AircraftPilot},
    aircraft_input::AircraftInputMgr,
    mesh_renderer::MeshInstancedRendererMgr,
    transform::TransformMgr,
};

pub async fn create(
    aircraft_mgr: &mut AircraftMgr,
    transform_mgr: &mut TransformMgr,
    aircraft_input_mgr: &mut AircraftInputMgr,
    render_state: &RenderState,
    mesh_renderer_mgr: &mut MeshInstancedRendererMgr,
) {
    // Load models
    let terrain_model = resources::load_model_obj(
        "models/Terrain_1.obj",
        &render_state.device,
        &render_state.queue,
        &mesh_renderer_mgr.texture_bind_group_layout,
    )
    .await
    .unwrap();

    let tank_model = resources::load_model_obj(
        "models/Tank_1.obj",
        &render_state.device,
        &render_state.queue,
        &mesh_renderer_mgr.texture_bind_group_layout,
    )
    .await
    .unwrap();

    // Positions and rotations
    let position_tank = cgmath::Vector3 {
        x: 0.0,
        y: 0.0,
        z: -50.0,
    };
    let rotation_tank =
        cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0));

    let position_terrain = cgmath::Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let rotation_terrain =
        cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0));

    // Player aircraft
    aircraft_mgr.add(
        AircraftPilot::Player,
        20.0,
        5.0,
        2.0,
        3.0,
        cgmath::Point3 {
            x: 0.0,
            y: 6.0,
            z: 10.0,
        },
        transform_mgr,
        aircraft_input_mgr,
    );

    // Add to mesh renderer manager
    mesh_renderer_mgr.add(
        render_state,
        terrain_model,
        position_terrain,
        rotation_terrain,
    );
    mesh_renderer_mgr.add(render_state, tank_model, position_tank, rotation_tank);
}
