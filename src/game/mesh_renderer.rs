use cgmath::{Quaternion, Vector3};
use wgpu::util::DeviceExt;

use crate::renderer::{
    model::{self, DrawModel},
    render_state::{create_render_pipeline, create_texture_bind_group_layout, RenderState},
    texture,
    vertex::Vertex,
};

use super::model::ModelMgr;

const MAX_MESH_COUNT: usize = 128;
const MAX_INSTANCE_COUNT: usize = 256;

// TODO: Currently the component supports just a single instance per mesh.
pub struct MeshInstancedRendererMgr {
    model_i: Vec<usize>,
    position: Vec<Vector3<f32>>,
    rotation: Vec<Quaternion<f32>>,

    instance_raw: Vec<Vec<model::InstanceRaw>>,
    instance_buffer: Vec<wgpu::Buffer>,

    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: wgpu::RenderPipeline,
}

impl MeshInstancedRendererMgr {
    pub fn new(render_state: &RenderState) -> Self {
        let model_i = Vec::with_capacity(MAX_MESH_COUNT);
        let position = Vec::with_capacity(MAX_MESH_COUNT);
        let rotation = Vec::with_capacity(MAX_MESH_COUNT);

        let instance_raw = Vec::with_capacity(MAX_MESH_COUNT);
        let instance_buffer = Vec::with_capacity(MAX_MESH_COUNT);

        let texture_bind_group_layout = create_texture_bind_group_layout(&render_state.device);
        let render_pipeline_layout =
            render_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Mesh render pipeline layout"),
                    bind_group_layouts: &[
                        &texture_bind_group_layout,
                        &render_state.camera_bind_group_layout,
                        &render_state.light_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let render_pipeline = {
            let shader_module_descriptor = wgpu::ShaderModuleDescriptor {
                label: Some("Mesh renderer shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../renderer/shaders/standard.wgsl").into(), // TODO: load shaders as resource
                ),
            };
            create_render_pipeline(
                &render_state.device,
                &render_pipeline_layout,
                render_state.config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), model::InstanceRaw::desc()],
                shader_module_descriptor,
                Some(wgpu::Face::Back),
            )
        };

        Self {
            model_i,
            position,
            rotation,

            instance_raw,
            instance_buffer,

            texture_bind_group_layout,
            render_pipeline,
        }
    }

    /// Returns instance index
    pub fn add(
        &mut self,
        render_state: &RenderState,
        model_i: usize,
        position: Vector3<f32>,
        rotation: Quaternion<f32>,
    ) -> usize {
        self.model_i.push(model_i);
        self.position.push(position);
        self.rotation.push(rotation);

        let index = self.len() - 1;

        let mut mesh_instances = Vec::with_capacity(MAX_INSTANCE_COUNT);
        mesh_instances.push(model::InstanceRaw::new(position, rotation));
        self.instance_raw.push(mesh_instances);

        let instance_buffer = self.create_instance_buffer(index, render_state);
        self.instance_buffer.push(instance_buffer);

        index
    }

    pub fn len(&self) -> usize {
        self.model_i.len()
    }

    fn create_instance_buffer(&self, index: usize, render_state: &RenderState) -> wgpu::Buffer {
        render_state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance buffer"),
                contents: bytemuck::cast_slice(&self.instance_raw[index]),
                usage: wgpu::BufferUsages::VERTEX,
            })
    }

    pub fn update_instance_position(
        &mut self,
        index: usize,
        position: Vector3<f32>,
        rotation: Quaternion<f32>,
        render_state: &RenderState,
    ) {
        self.instance_raw[index][0].update(position, rotation);
        // TODO: use queue.write_buffer instead of recreating the buffer
        let instance_buffer = self.create_instance_buffer(index, render_state);
        self.instance_buffer[index] = instance_buffer;
    }

    pub fn render(
        &mut self,
        model_mgr: &ModelMgr,
        render_state: &RenderState,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) -> Result<(), wgpu::SurfaceError> {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Mesh render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(render_state.clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &render_state.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        render_pass.set_pipeline(&self.render_pipeline);

        for i in 0..self.len() {
            let model_i = self.model_i[i];
            let model = &model_mgr.model[model_i];

            render_pass.set_vertex_buffer(1, self.instance_buffer[i].slice(..));

            render_pass.draw_model_instanced(
                model,
                0..self.instance_raw[i].len() as u32,
                &render_state.camera_bind_group,
                &render_state.light_bind_group,
            );
        }

        Ok(())
    }
}
