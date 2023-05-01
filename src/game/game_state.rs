use std::time::Duration;

use winit::{event::Event, event_loop::EventLoop, window::Window};

use crate::{
    audio::audio_manager::AudioMgr,
    input::{cursor_manager::CursorMgr, keyboard_manager::KeyboardMgr},
    renderer::render_state::RenderState,
};

use super::{
    aircraft::AircraftMgr, aircraft_input::AircraftInputMgr, audio_test::AudioTest,
    camera::player_camera::PlayerCameraController, egui_manager::egui_renderer::EguiRenderer,
    mesh_renderer::MeshInstancedRendererMgr, on_screen_diagnostics::OnScreenDiagnostics,
    sample_scene, transform::TransformMgr,
};

pub struct GameState {
    cursor_mgr: CursorMgr,
    keyboard_mgr: KeyboardMgr,

    player_camera: PlayerCameraController,

    aircraft_mgr: AircraftMgr,
    aircraft_input_mgr: AircraftInputMgr,
    transform_mgr: TransformMgr,

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

        let player_camera = PlayerCameraController::new();

        let egui_renderer = EguiRenderer::new(event_loop, render_state);
        let on_screen_diagnostics = OnScreenDiagnostics::new(0.1);
        let mut mesh_instanced_renderer_mgr = MeshInstancedRendererMgr::new(render_state);
        let audio_mgr = AudioMgr::new();

        let audio_test = AudioTest::new().await;

        let mut aircraft_mgr = AircraftMgr::new().unwrap();
        let mut aircraft_input_mgr = AircraftInputMgr::new();
        let mut transform_mgr = TransformMgr::new();

        sample_scene::create(
            &mut aircraft_mgr,
            &mut transform_mgr,
            &mut aircraft_input_mgr,
            render_state,
            &mut mesh_instanced_renderer_mgr,
        )
        .await;

        Self {
            cursor_mgr,
            keyboard_mgr,

            player_camera,

            aircraft_mgr,
            aircraft_input_mgr,
            transform_mgr,

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
    }

    /// Handle component updates
    pub fn update(&mut self, render_state: &mut RenderState, dt: Duration) {
        self.aircraft_input_mgr.update(&self.keyboard_mgr);
        self.aircraft_mgr.update(
            &mut self.transform_mgr,
            &mut self.aircraft_input_mgr,
            &mut self.mesh_instanced_renderer_mgr,
            &render_state,
            dt,
        );

        self.transform_mgr.update();

        self.player_camera.update(
            &mut render_state.camera,
            &self.aircraft_mgr,
            &self.transform_mgr,
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
