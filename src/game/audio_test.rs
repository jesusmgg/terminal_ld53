use kira::sound::static_sound::{PlaybackState, StaticSoundData, StaticSoundHandle};
use winit::event::{ElementState, MouseButton};

use crate::{audio::audio_manager::AudioMgr, resources};

pub struct AudioTest {
    sound_data: StaticSoundData,
    playing_sound_data: Option<StaticSoundHandle>,

    mouse_pressed: bool,
}

impl AudioTest {
    pub async fn new() -> Self {
        let sound_data = resources::load_static_sound_data("audio/Cursor_tones/cursor_style_2.ogg")
            .await
            .unwrap();

        Self {
            sound_data,
            playing_sound_data: None,
            mouse_pressed: false,
        }
    }

    pub fn input<T>(&mut self, event: &winit::event::Event<T>, window: &winit::window::Window) {
        match *event {
            winit::event::Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                winit::event::WindowEvent::MouseInput {
                    state,
                    button: MouseButton::Left,
                    ..
                } => {
                    self.mouse_pressed = *state == ElementState::Pressed;
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub fn update(&mut self, audio_mgr: &mut AudioMgr) {
        let is_playing = match &self.playing_sound_data {
            Some(sound_handle) => sound_handle.state() == PlaybackState::Playing,
            None => false,
        };

        if self.mouse_pressed && !is_playing {
            self.playing_sound_data = Some(audio_mgr.play(self.sound_data.clone()).unwrap());
        }
    }
}
