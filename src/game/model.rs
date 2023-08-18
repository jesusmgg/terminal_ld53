use crate::{
    renderer::{model::Model, render_state::RenderState},
    resources,
};

use super::mesh_renderer::MeshInstancedRendererMgr;

const MAX_INSTANCE_COUNT: usize = 128;

pub struct ModelMgr {
    pub name: Vec<String>,
    pub model: Vec<Model>,
}

impl ModelMgr {
    pub fn new() -> Self {
        let name = Vec::with_capacity(MAX_INSTANCE_COUNT);
        let model = Vec::with_capacity(MAX_INSTANCE_COUNT);
        Self { name, model }
    }

    pub async fn add(
        &mut self,
        model_path: &str,
        render_state: &RenderState,
        mesh_renderer_mgr: &MeshInstancedRendererMgr,
    ) -> usize {
        let name = String::from(model_path);
        self.name.push(name);

        let model = resources::load_model_obj(
            model_path,
            &render_state.device,
            &render_state.queue,
            &mesh_renderer_mgr.texture_bind_group_layout,
        )
        .await
        .unwrap();

        self.model.push(model);

        let index = self.len() - 1;
        index
    }

    pub fn len(&self) -> usize {
        self.model.len()
    }

    pub fn get_with_name(&self, name: &str) -> Option<usize> {
        for (index, model_name) in self.name.iter().enumerate() {
            if model_name.eq(name) {
                return Some(index);
            }
        }

        None
    }

    pub async fn get_with_name_or_add(
        &mut self,
        model_path: &str,
        render_state: &RenderState,
        mesh_renderer_mgr: &MeshInstancedRendererMgr,
    ) -> usize {
        let index = match self.get_with_name(&model_path) {
            Some(index) => index,
            None => self.add(&model_path, render_state, mesh_renderer_mgr).await,
        };

        index
    }
}
