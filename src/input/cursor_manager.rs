use winit::{
    dpi::LogicalPosition,
    event::{Event, WindowEvent},
};

pub struct CursorMgr {
    pub x: f32,
    pub y: f32,
}

impl CursorMgr {
    pub fn new(window: &mut winit::window::Window) -> CursorMgr {
        // Center cursor on start
        let window_size = window.inner_size();
        window
            .set_cursor_position(LogicalPosition {
                x: window_size.width / 2,
                y: window_size.height / 2,
            })
            .unwrap();

        Self { x: 0.0, y: 0.0 }
    }

    pub fn input<T>(&mut self, event: &winit::event::Event<T>, window: &winit::window::Window) {
        match *event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CursorMoved { position, .. } => {
                    self.x = position.x as f32;
                    self.y = position.y as f32;
                }
                _ => (),
            },
            _ => (),
        };
    }
}
