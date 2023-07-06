use anyhow::Result;
use cgmath::{EuclideanSpace, Point3, Quaternion};

use crate::{
    game::{
        inventory::InventoryMgr, mesh_renderer::MeshInstancedRendererMgr, transform::TransformMgr,
    },
    renderer::render_state::RenderState,
    resources,
};

const MAX_INSTANCE_COUNT: usize = 128;

pub struct BuildingMgr {
    pub building_type: Vec<BuildingType>,

    pub supply_range: Vec<Option<f32>>,

    pub inventory_i: Vec<Option<usize>>,

    pub transform_i: Vec<Option<usize>>,
    pub mesh_renderer_i: Vec<Option<usize>>,
}

impl BuildingMgr {
    pub fn new() -> Self {
        Self {
            building_type: Vec::with_capacity(MAX_INSTANCE_COUNT),
            supply_range: Vec::with_capacity(MAX_INSTANCE_COUNT),

            inventory_i: Vec::with_capacity(MAX_INSTANCE_COUNT),

            transform_i: Vec::with_capacity(MAX_INSTANCE_COUNT),
            mesh_renderer_i: Vec::with_capacity(MAX_INSTANCE_COUNT),
        }
    }

    pub async fn add(
        &mut self,

        building_type: BuildingType,

        supply_range: Option<f32>,

        position: Point3<f32>,
        rotation: Quaternion<f32>,

        inventory_mgr: &mut InventoryMgr,

        transform_mgr: &mut TransformMgr,
        mesh_renderer_mgr: &mut MeshInstancedRendererMgr,
        render_state: &RenderState,
    ) -> Result<usize> {
        self.building_type.push(building_type.clone());

        self.inventory_i.push(Some(inventory_mgr.add().unwrap()));

        self.supply_range.push(supply_range);

        self.transform_i
            .push(Some(transform_mgr.add(position, rotation)));

        let factory_model = resources::load_model_obj(
            "models/cube.obj",
            &render_state.device,
            &render_state.queue,
            &mesh_renderer_mgr.texture_bind_group_layout,
        )
        .await
        .unwrap();

        let mesh_renderer_i =
            Some(mesh_renderer_mgr.add(render_state, factory_model, position.to_vec(), rotation));
        self.mesh_renderer_i.push(mesh_renderer_i);

        let index = self.len() - 1;
        Ok(index)
    }

    pub fn len(&self) -> usize {
        self.building_type.len()
    }
}

#[derive(Clone)]
pub enum BuildingType {
    Factory,
}
