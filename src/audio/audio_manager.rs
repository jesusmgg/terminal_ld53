use kira::{
    manager::{self, backend::cpal::CpalBackend, error::PlaySoundError, AudioManagerSettings},
    sound::SoundData,
};

pub struct AudioMgr {
    manager: manager::AudioManager,
}

impl AudioMgr {
    pub fn new() -> Self {
        let manager_settings = AudioManagerSettings::default();
        let manager = manager::AudioManager::<CpalBackend>::new(manager_settings).unwrap();
        Self { manager }
    }

    pub fn play<D: SoundData>(
        &mut self,
        sound_data: D,
    ) -> Result<D::Handle, PlaySoundError<D::Error>> {
        self.manager.play(sound_data)
    }
}
