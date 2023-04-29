use cgmath::Rotation3;

use crate::{renderer::render_state::RenderState, resources};

use super::mesh_renderer::MeshInstancedRendererMgr;

pub async fn create(render_state: &RenderState, mesh_renderer_mgr: &mut MeshInstancedRendererMgr) {
    // glTF test
    let gltf_tank_model = resources::load_model_gltf(
        "models/Tank_1.gltf",
        &render_state.device,
        &render_state.queue,
        &mesh_renderer_mgr.texture_bind_group_layout,
    )
    .await
    .unwrap();

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

    let cube_model = resources::load_model_obj(
        "models/cube.obj",
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

    let position_cube = cgmath::Vector3 {
        x: 10.0,
        y: 10.0,
        z: -25.0,
    };
    let rotation_cube =
        cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0));

    let position_terrain = cgmath::Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let rotation_terrain =
        cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0));

    // Add to mesh renderer manager
    mesh_renderer_mgr.add(
        render_state,
        terrain_model,
        position_terrain,
        rotation_terrain,
    );
    mesh_renderer_mgr.add(render_state, gltf_tank_model, position_tank, rotation_tank);
    mesh_renderer_mgr.add(render_state, cube_model, position_cube, rotation_cube);
}
