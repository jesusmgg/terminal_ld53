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
        // let i = aircraft_mgr.get_player_aircraft_index();
        let i = 1;
        let transform_i = aircraft_mgr.transform_i[i].unwrap();

        camera.set_from_transform_mgr(transform_mgr, transform_i);
    }
}
