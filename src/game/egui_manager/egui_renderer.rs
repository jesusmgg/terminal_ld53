use egui;
use egui_winit::pixels_per_point;
use winit::{event::Event, event_loop::EventLoop, window::Window};

use crate::renderer::{render_state::RenderState, texture};

pub struct EguiRenderer {
    pub context: egui::Context,
    platform: egui_winit::State,
    renderer: egui_wgpu::Renderer,
    full_output: Option<egui::FullOutput>,
}

impl EguiRenderer {
    pub fn new<T>(event_loop: &EventLoop<T>, render_state: &RenderState) -> Self {
        let renderer = egui_wgpu::Renderer::new(
            &render_state.device,
            render_state.config.format,
            Some(texture::Texture::DEPTH_FORMAT),
            1,
        );

        let context = egui::Context::default();

        let platform = egui_winit::State::new(
            context.viewport_id(),
            event_loop,
            Some(pixels_per_point(&context, &render_state.window)),
            Some(1024),
        );

        Self {
            context,
            platform,
            renderer,
            full_output: None,
        }
    }

    pub fn input<T>(&mut self, event: &winit::event::Event<T>, window: &winit::window::Window) {
        match *event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                let _ = self.platform.on_window_event(&self.context, event);
            }
            _ => (),
        }
    }

    pub fn ui_begin_frame(&mut self, window: &Window) {
        let raw_input = self.platform.take_egui_input(window);
        self.context.begin_frame(raw_input);
    }

    pub fn ui_end_frame(&mut self) {
        self.full_output = Some(self.context.end_frame());
    }

    pub fn render(
        &mut self,
        render_state: &RenderState,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) -> Result<(), wgpu::SurfaceError> {
        if self.full_output.is_some() {
            let full_output = self.full_output.as_ref().unwrap();

            self.platform.handle_platform_output(
                &render_state.window,
                &self.context,
                full_output.platform_output.clone(),
            );

            let pixels_per_point = pixels_per_point(&self.context, &render_state.window);

            let paint_jobs = self
                .context
                .tessellate(full_output.shapes.clone(), pixels_per_point);

            let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
                size_in_pixels: [render_state.config.width, render_state.config.height],
                pixels_per_point,
            };

            for (id, image_delta) in &full_output.textures_delta.set {
                self.renderer.update_texture(
                    &render_state.device,
                    &render_state.queue,
                    *id,
                    image_delta,
                );
            }

            self.renderer.update_buffers(
                &render_state.device,
                &render_state.queue,
                encoder,
                &paint_jobs,
                &screen_descriptor,
            );

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
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

            self.renderer
                .render(&mut render_pass, &paint_jobs, &screen_descriptor);
        }

        Ok(())
    }
}
