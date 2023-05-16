use std::time::Duration;

pub struct OnScreenDiagnostics {
    frame_time: f32,
    fps: f32,

    update_period: f32,
    update_timer: f32,
}

impl OnScreenDiagnostics {
    pub fn new(update_period: f32) -> Self {
        Self {
            frame_time: 0.0,
            fps: 0.0,
            update_period,
            update_timer: update_period,
        }
    }

    pub fn update(&mut self, dt: Duration) {
        let dt = dt.as_secs_f32();

        self.update_timer += dt;

        if self.update_timer >= self.update_period {
            self.fps = 1.0 / dt;
            self.frame_time = dt * 1000.0;

            self.update_timer = 0.0;
        }
    }

    pub fn ui(&self, context: &egui::Context) {
        let title_str = format!("Diagnostics (every {}s):", self.update_period);
        let frame_time_str = format!("FT: {}ms", self.frame_time);
        let fps_str = format!("FPS: {}", self.fps);

        egui::SidePanel::right("right_panel")
            .resizable(false)
            .min_width(150.0)
            .show(context, |ui| {
                ui.label(title_str);
                ui.label("----------------------");
                ui.label(frame_time_str);
                ui.label(fps_str);
            });
    }
}
