use std::time::Duration;

use winit::{event::Event, event_loop::EventLoop, window::Window};

use crate::{
    audio::audio_manager::AudioMgr,
    input::{cursor_manager::CursorMgr, keyboard_manager::KeyboardMgr},
    renderer::render_state::RenderState,
};

use super::{
    audio_test::AudioTest, camera::rts_camera::RtsCameraController,
    egui_manager::egui_renderer::EguiRenderer, mesh_renderer::MeshInstancedRendererMgr,
    on_screen_diagnostics::OnScreenDiagnostics, sample_scene,
};

pub struct GameState {
    cursor_mgr: CursorMgr,
    keyboard_mgr: KeyboardMgr,

    camera_controller: RtsCameraController,
    egui_renderer: EguiRenderer,
    on_screen_diagnostics: OnScreenDiagnostics,
    mesh_instanced_renderer_mgr: MeshInstancedRendererMgr,
    audio_mgr: AudioMgr,

    audio_test: AudioTest,
}

impl GameState {
    pub async fn new<T>(event_loop: &EventLoop<T>, render_state: &mut RenderState) -> GameState {
        let cursor_mgr = CursorMgr::new(&mut render_state.window);
        let keyboard_mgr = KeyboardMgr::new();

        let camera_controller =
            RtsCameraController::new(70.0, 40.0, 10.0, 40.0, -50.0, &mut render_state.camera);
        let egui_renderer = EguiRenderer::new(event_loop, render_state);
        let on_screen_diagnostics = OnScreenDiagnostics::new(0.1);
        let mut mesh_instanced_renderer_mgr = MeshInstancedRendererMgr::new(render_state);
        let audio_mgr = AudioMgr::new();

        let audio_test = AudioTest::new().await;
        sample_scene::create(render_state, &mut mesh_instanced_renderer_mgr).await;

        Self {
            cursor_mgr,
            keyboard_mgr,

            camera_controller,
            egui_renderer,
            on_screen_diagnostics,
            mesh_instanced_renderer_mgr,
            audio_mgr,

            audio_test,
        }
    }

    /// Handle component inputs
    /// Returs `true` if any event has been processed.
    pub fn input<T>(&mut self, event: &Event<T>, window: &Window) {
        self.cursor_mgr.input(event, window);
        self.keyboard_mgr.input(event, window);

        self.egui_renderer.input(event, window);
        self.audio_test.input(event, window);

        self.camera_controller.input(&self.keyboard_mgr);
    }

    /// Handle component updates
    pub fn update(&mut self, render_state: &mut RenderState, dt: Duration) {
        self.camera_controller.update(
            &mut render_state.camera,
            &self.cursor_mgr,
            &self.keyboard_mgr,
            &render_state.window,
            dt,
        );
        self.audio_test.update(&mut self.audio_mgr);
        self.on_screen_diagnostics.update(dt);
    }

    /// Handle component UI layout
    pub fn ui(&mut self, render_state: &mut RenderState) {
        self.egui_renderer.ui_begin_frame(&render_state.window);

        self.on_screen_diagnostics.ui(&self.egui_renderer.context);

        self.egui_renderer.ui_end_frame();
    }

    /// Handle component renders
    pub fn render(
        &mut self,
        render_state: &RenderState,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        self.mesh_instanced_renderer_mgr
            .render(render_state, encoder, view)
            .unwrap();
        self.egui_renderer
            .render(render_state, encoder, view)
            .unwrap();
    }
}
