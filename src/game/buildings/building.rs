use anyhow::Result;
use cgmath::{EuclideanSpace, Point3, Quaternion};

use crate::{
    game::{
        inventory::InventoryMgr, mesh_renderer::MeshInstancedRendererMgr, model::ModelMgr,
        transform::TransformMgr,
    },
    renderer::render_state::RenderState,
};

const MAX_INSTANCE_COUNT: usize = 128;

pub struct BuildingMgr {
    pub building_type: Vec<BuildingType>,

    pub supply_range: Vec<Option<f32>>,
    pub supply_period_ms: Vec<Option<u32>>,

    pub inventory_i: Vec<Option<usize>>,

    pub transform_i: Vec<Option<usize>>,
    pub mesh_renderer_i: Vec<Option<usize>>,
}

impl BuildingMgr {
    pub fn new() -> Self {
        Self {
            building_type: Vec::with_capacity(MAX_INSTANCE_COUNT),
            supply_range: Vec::with_capacity(MAX_INSTANCE_COUNT),
            supply_period_ms: Vec::with_capacity(MAX_INSTANCE_COUNT),

            inventory_i: Vec::with_capacity(MAX_INSTANCE_COUNT),

            transform_i: Vec::with_capacity(MAX_INSTANCE_COUNT),
            mesh_renderer_i: Vec::with_capacity(MAX_INSTANCE_COUNT),
        }
    }

    pub async fn add(
        &mut self,

        building_type: BuildingType,

        supply_range: Option<f32>,
        supply_period_ms: Option<u32>,

        position: Point3<f32>,
        rotation: Quaternion<f32>,

        inventory_mgr: &mut InventoryMgr,

        transform_mgr: &mut TransformMgr,
        model_mgr: &mut ModelMgr,
        mesh_renderer_mgr: &mut MeshInstancedRendererMgr,
        render_state: &RenderState,
    ) -> Result<usize> {
        self.building_type.push(building_type.clone());

        self.inventory_i.push(Some(inventory_mgr.add().unwrap()));

        self.supply_range.push(supply_range);
        self.supply_period_ms.push(supply_period_ms);

        self.transform_i
            .push(Some(transform_mgr.add(position, rotation)));

        let model_i = model_mgr
            .get_with_name_or_add("models/cube.obj", render_state, mesh_renderer_mgr)
            .await;

        let mesh_renderer_i =
            Some(mesh_renderer_mgr.add(render_state, model_i, position.to_vec(), rotation));
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
