use cgmath::{EuclideanSpace, Point3, Quaternion};

use crate::{
    collision::collider::ColliderMgr,
    game::{mesh_renderer::MeshInstancedRendererMgr, transform::TransformMgr},
    renderer::render_state::RenderState,
    resources,
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
        mesh_renderer_mgr: &mut MeshInstancedRendererMgr,
        render_state: &RenderState,
    ) -> Self {
        let transform_i = transform_mgr.add(position, rotation);

        let terrain_model = resources::load_model_obj(
            model_path,
            &render_state.device,
            &render_state.queue,
            &mesh_renderer_mgr.texture_bind_group_layout,
        )
        .await
        .unwrap();

        let collider_i = collider_mgr
            .add_from_model(&terrain_model, transform_i)
            .unwrap();

        let mesh_renderer_i =
            mesh_renderer_mgr.add(render_state, terrain_model, position.to_vec(), rotation);

        Self {
            transform_i,
            collider_i,
            mesh_renderer_i,
        }
    }
}
