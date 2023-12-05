use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    window::Window,
};

const KEYCODE_COUNT: usize = 512;

// TODO: Update to support new key event enum (NamedKey, Character)
pub struct KeyboardMgr {
    pub key_pressed: [bool; KEYCODE_COUNT],
    pub key_down: [bool; KEYCODE_COUNT],
    pub key_up: [bool; KEYCODE_COUNT],
}

impl KeyboardMgr {
    pub fn new() -> Self {
        let key_pressed = [false; KEYCODE_COUNT];
        let key_down = [false; KEYCODE_COUNT];
        let key_up = [false; KEYCODE_COUNT];

        Self {
            key_pressed,
            key_down,
            key_up,
        }
    }

    pub fn input<T>(&mut self, event: &Event<T>, window: &Window) {
        match *event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(key),
                            state,
                            ..
                        },
                    ..
                } => self.update_key_state(*key, *state),
                _ => (),
            },
            _ => (),
        }
    }

    // TODO: make key down and up more responsive, consider clearing the state after each update
    fn update_key_state(&mut self, key: VirtualKeyCode, state: ElementState) {
        let key = key as usize;
        let is_pressed = state == ElementState::Pressed;

        let pressed_prev_frame = self.key_pressed[key];

        let mut is_down = false;
        let mut is_up = false;

        if !pressed_prev_frame && is_pressed {
            is_down = true;
        } else if pressed_prev_frame && !is_pressed {
            is_up = true;
        }

        self.key_pressed[key] = is_pressed;
        self.key_down[key] = is_down;
        self.key_up[key] = is_up;
    }
}
