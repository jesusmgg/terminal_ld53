use crate::{
    game::{aircraft::AircraftMgr, transform::TransformMgr},
    renderer::camera::Camera,
};

pub struct PlayerCameraController {}

impl PlayerCameraController {
    pub fn new() -> PlayerCameraController {
        PlayerCameraController {}
    }

    pub fn update(
        &mut self,
        camera: &mut Camera,
        aircraft_mgr: &AircraftMgr,
        transform_mgr: &TransformMgr,
    ) {
        let i = aircraft_mgr.get_player_aircraft_index();
        let transform_i = aircraft_mgr.transform_i[i];

        camera.set(
            transform_mgr.position[transform_i],
            transform_mgr.yaw[transform_i],
            transform_mgr.pitch[transform_i],
            transform_mgr.roll[transform_i],
        );
    }
}
