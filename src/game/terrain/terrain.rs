use cgmath::{EuclideanSpace, Point3, Quaternion};

use crate::{
    game::collision::collider::ColliderMgr,
    game::{mesh_renderer::MeshInstancedRendererMgr, model::ModelMgr, transform::TransformMgr},
    renderer::render_state::RenderState,
};

pub struct Terrain {
    transform_i: usize,
    collider_i: usize,
    mesh_renderer_i: usize,
}

impl Terrain {
    pub async fn new(
        model_path: &str,
        position: Point3<f32>,
        rotation: Quaternion<f32>,
        transform_mgr: &mut TransformMgr,
        collider_mgr: &mut ColliderMgr,
        model_mgr: &mut ModelMgr,
        mesh_renderer_mgr: &mut MeshInstancedRendererMgr,
        render_state: &RenderState,
    ) -> Self {
        let transform_i = transform_mgr.add(position, rotation);

        let model_i = model_mgr
            .add(model_path, &render_state, &mesh_renderer_mgr)
            .await;

        let collider_i = collider_mgr
            .add_from_model(model_i, transform_i, &model_mgr)
            .unwrap();

        let mesh_renderer_i =
            mesh_renderer_mgr.add(render_state, model_i, position.to_vec(), rotation);

        Self {
            transform_i,
            collider_i,
            mesh_renderer_i,
        }
    }
}
