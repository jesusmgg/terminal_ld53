use std::ops::Range;

use wgpu::util::DeviceExt;

use crate::renderer::{
    model::{self, DrawModel},
    render_state::{create_render_pipeline, create_texture_bind_group_layout, RenderState},
    texture,
    vertex::Vertex,
};

const MAX_MESH_COUNT: usize = 128;
const MAX_INSTANCE_COUNT: usize = 256;

// TODO: position/rotation should probably be independtly managed.
//       This component should take just about rendering as much as possible.
// TODO: add mesh instance update facilities.
//       Currently the component supports just a single instance per mesh.
pub struct MeshInstancedRendererMgr {
    model: Vec<model::Model>,
    position: Vec<cgmath::Vector3<f32>>,
    rotation: Vec<cgmath::Quaternion<f32>>,

    instance_raw: Vec<Vec<model::InstanceRaw>>,
    instance_buffer: Vec<wgpu::Buffer>,
    is_instance_buffer_dirty: Vec<bool>,

    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: wgpu::RenderPipeline,
}

impl MeshInstancedRendererMgr {
    pub fn new(render_state: &RenderState) -> Self {
        let model = Vec::with_capacity(MAX_MESH_COUNT);
        let position = Vec::with_capacity(MAX_MESH_COUNT);
        let rotation = Vec::with_capacity(MAX_MESH_COUNT);

        let instance_raw = Vec::with_capacity(MAX_MESH_COUNT);
        let instance_buffer = Vec::with_capacity(MAX_MESH_COUNT);
        let is_instance_buffer_dirty = Vec::with_capacity(MAX_MESH_COUNT);

        let texture_bind_group_layout = create_texture_bind_group_layout(&render_state.device);
        let render_pipeline_layout =
            render_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render pipeline layout"),
                    bind_group_layouts: &[
                        &texture_bind_group_layout,
                        &render_state.camera_bind_group_layout,
                        &render_state.light_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let render_pipeline = {
            let shader_module_descriptor = wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../renderer/standard_shader.wgsl").into(), // TODO: load as resource
                ),
            };
            create_render_pipeline(
                &render_state.device,
                &render_pipeline_layout,
                render_state.config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), model::InstanceRaw::desc()],
                shader_module_descriptor,
            )
        };

        Self {
            model,
            position,
            rotation,

            instance_raw,
            instance_buffer,
            is_instance_buffer_dirty,

            texture_bind_group_layout,
            render_pipeline,
        }
    }

    /// Returns instance index
    pub fn add(
        &mut self,
        render_state: &RenderState,
        model: model::Model,
        position: cgmath::Vector3<f32>,
        rotation: cgmath::Quaternion<f32>,
    ) -> usize {
        self.model.push(model);
        self.position.push(position);
        self.rotation.push(rotation);

        let mut mesh_instances = Vec::with_capacity(MAX_INSTANCE_COUNT);
        mesh_instances.push(model::InstanceRaw::new(position, rotation));

        let instance_buffer =
            render_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instance buffer"),
                    contents: bytemuck::cast_slice(&mesh_instances),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        self.instance_buffer.push(instance_buffer);
        self.is_instance_buffer_dirty.push(false);

        self.instance_raw.push(mesh_instances);

        self.model.len() - 1
    }

    /// Returns appended instance index range
    pub fn append(
        &mut self,
        render_state: &RenderState,
        mut models: Vec<model::Model>,
        mut positions: Vec<cgmath::Vector3<f32>>,
        mut rotations: Vec<cgmath::Quaternion<f32>>,
    ) -> Option<Range<usize>> {
        let new_model_count = models.len();
        if new_model_count == 0
            || new_model_count != positions.len()
            || new_model_count != rotations.len()
        {
            return None;
        }

        let start = self.model.len() - 1;

        self.model.append(&mut models);
        self.position.append(&mut positions);
        self.rotation.append(&mut rotations);

        let end = self.model.len();

        for i in start..end {
            let mut mesh_instances = Vec::with_capacity(MAX_INSTANCE_COUNT);
            mesh_instances.push(model::InstanceRaw::new(positions[i], rotations[i]));

            let instance_buffer =
                render_state
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Instance buffer"),
                        contents: bytemuck::cast_slice(&mesh_instances),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
            self.instance_buffer.push(instance_buffer);
            self.is_instance_buffer_dirty.push(false);

            self.instance_raw.push(mesh_instances);
        }

        Some(Range { start, end })
    }

    pub fn render(
        &mut self,
        render_state: &RenderState,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) -> Result<(), wgpu::SurfaceError> {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(render_state.clear_color),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &render_state.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });
        render_pass.set_pipeline(&self.render_pipeline);

        for i in 0..self.model.len() {
            // TODO: Update dirty instance buffers

            render_pass.set_vertex_buffer(1, self.instance_buffer[i].slice(..));

            render_pass.draw_model_instanced(
                &self.model[i],
                0..self.instance_raw[i].len() as u32,
                &render_state.camera_bind_group,
                &render_state.light_bind_group,
            );
        }

        Ok(())
    }
}
