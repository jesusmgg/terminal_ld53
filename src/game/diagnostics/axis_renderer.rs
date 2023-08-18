use wgpu::util::DeviceExt;

use crate::renderer::{
    render_state::{create_render_pipeline, RenderState},
    texture,
    vertex::Vertex,
};

use super::axis_vertex::{AxisVertex, AXIS_VERTICES};

/// Renders coordinates axis.
// TODO: support multiple instances. This may need a `position: Vec<Point3<f32>>`
pub struct AxisRendererMgr {
    vertex_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,
}

impl AxisRendererMgr {
    pub fn new(render_state: &RenderState) -> Self {
        let vertex_buffer =
            render_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Axis vertex buffer"),
                    contents: bytemuck::cast_slice(&AXIS_VERTICES),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });

        // TODO: probably redundant
        render_state
            .queue
            .write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(&AXIS_VERTICES));

        let render_pipeline_layout =
            render_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Axis render pipeline layout"),
                    bind_group_layouts: &[&render_state.camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let render_pipeline = {
            let shader_module_descriptor = wgpu::ShaderModuleDescriptor {
                label: Some("Axis shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../../renderer/shaders/axis.wgsl").into(), // TODO: load shaders as resource
                ),
            };
            create_render_pipeline(
                &render_state.device,
                &render_pipeline_layout,
                render_state.config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[AxisVertex::desc()],
                shader_module_descriptor,
                None,
            )
        };

        Self {
            vertex_buffer,
            render_pipeline,
        }
    }

    pub fn render(
        &mut self,
        render_state: &RenderState,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) -> anyhow::Result<(), wgpu::SurfaceError> {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Axis render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
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
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_bind_group(0, &render_state.camera_bind_group, &[]);
        render_pass.draw(0..AXIS_VERTICES.len() as u32, 0..1);

        Ok(())
    }
}
