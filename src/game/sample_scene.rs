use cgmath::Rotation3;

use crate::{renderer::render_state::RenderState, resources};

use super::{
    aircraft::{AircraftMgr, AircraftPilot},
    aircraft_input::AircraftInputMgr,
    buildings::building::{BuildingMgr, BuildingType},
    inventory::InventoryMgr,
    mesh_renderer::MeshInstancedRendererMgr,
    transform::TransformMgr,
};

pub async fn create(
    aircraft_mgr: &mut AircraftMgr,
    transform_mgr: &mut TransformMgr,
    aircraft_input_mgr: &mut AircraftInputMgr,
    building_mgr: &mut BuildingMgr,
    inventory_mgr: &mut InventoryMgr,
    render_state: &RenderState,
    mesh_renderer_mgr: &mut MeshInstancedRendererMgr,
) {
    // Load terrain
    let terrain_model = resources::load_model_obj(
        "models/Terrain_1.obj",
        &render_state.device,
        &render_state.queue,
        &mesh_renderer_mgr.texture_bind_group_layout,
    )
    .await
    .unwrap();

    let position_terrain = cgmath::Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let rotation_terrain =
        cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0));

    mesh_renderer_mgr.add(
        render_state,
        terrain_model,
        position_terrain,
        rotation_terrain,
    );

    // Player aircraft
    aircraft_mgr
        .add(
            AircraftPilot::Player,
            6.0,
            0.0,
            5.0,
            2.0,
            5.0,
            3.0,
            6.0,
            cgmath::Point3 {
                x: 0.0,
                y: 6.0,
                z: 10.0,
            },
            cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0)),
            inventory_mgr,
            transform_mgr,
            aircraft_input_mgr,
            mesh_renderer_mgr,
            render_state,
        )
        .await
        .unwrap();

    // Enemy aircraft
    aircraft_mgr
        .add(
            AircraftPilot::Ai,
            20.0,
            10.0,
            5.0,
            5.0,
            10.0,
            3.0,
            6.0,
            cgmath::Point3 {
                x: 30.0,
                y: 6.0,
                z: 30.0,
            },
            cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0)),
            inventory_mgr,
            transform_mgr,
            aircraft_input_mgr,
            mesh_renderer_mgr,
            render_state,
        )
        .await
        .unwrap();

    let mut rng = oorandom::Rand32::new(1234);

    for i in 0..10 {
        aircraft_mgr
            .add(
                AircraftPilot::Ai,
                20.0,
                10.0,
                5.0,
                5.0,
                10.0,
                3.0,
                6.0,
                cgmath::Point3 {
                    x: rng.rand_range(0..1000) as f32 - 500.0,
                    y: rng.rand_range(10..100) as f32,
                    z: rng.rand_range(0..1000) as f32 - 500.0,
                },
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0)),
                inventory_mgr,
                transform_mgr,
                aircraft_input_mgr,
                mesh_renderer_mgr,
                render_state,
            )
            .await
            .unwrap();
    }

    // Buildings
    building_mgr
        .add(
            BuildingType::Factory,
            Some(10.0),
            cgmath::Point3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0)),
            inventory_mgr,
            transform_mgr,
            mesh_renderer_mgr,
            render_state,
        )
        .await
        .unwrap();
}
