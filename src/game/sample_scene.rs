use cgmath::Rotation3;

use crate::{collision::collider::ColliderMgr, renderer::render_state::RenderState};

use super::{
    aircraft::{AircraftMgr, AircraftPilot},
    aircraft_input::AircraftInputMgr,
    buildings::building::{BuildingMgr, BuildingType},
    inventory::InventoryMgr,
    mesh_renderer::MeshInstancedRendererMgr,
    terrain::terrain::Terrain,
    transform::TransformMgr,
};

pub async fn create(
    aircraft_mgr: &mut AircraftMgr,
    transform_mgr: &mut TransformMgr,
    collider_mgr: &mut ColliderMgr,
    aircraft_input_mgr: &mut AircraftInputMgr,
    building_mgr: &mut BuildingMgr,
    inventory_mgr: &mut InventoryMgr,
    render_state: &RenderState,
    mesh_renderer_mgr: &mut MeshInstancedRendererMgr,
) {
    // Load terrain
    let position_terrain = cgmath::Point3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let rotation_terrain =
        cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0));

    let _terrain = Terrain::new(
        "models/Terrain_1.obj",
        position_terrain,
        rotation_terrain,
        transform_mgr,
        collider_mgr,
        mesh_renderer_mgr,
        &render_state,
    )
    .await;

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
            collider_mgr,
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
            1.0,
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
            cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_x(), cgmath::Deg(00.0)),
            inventory_mgr,
            transform_mgr,
            collider_mgr,
            aircraft_input_mgr,
            mesh_renderer_mgr,
            render_state,
        )
        .await
        .unwrap();

    let mut rng = oorandom::Rand32::new(1234);

    for _ in 0..10 {
        aircraft_mgr
            .add(
                AircraftPilot::Ai,
                20.0,
                1.0,
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
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(0.0)),
                inventory_mgr,
                transform_mgr,
                collider_mgr,
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
            Some(1000),
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
