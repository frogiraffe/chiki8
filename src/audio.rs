use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired, AudioStatus};
use sdl2::Sdl;

pub struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            *x = if self.phase < 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

pub struct Sound {
    pub device: AudioDevice<SquareWave>,
}

impl Sound {
    pub fn new(sdl_context: &Sdl, volume: f32) -> Result<Sound, String> {
        let audio_subsystem = sdl_context
            .audio()
            .map_err(|error| format!("SDL audio initialization failed: {error}"))?;
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: None,
        };
        let device = audio_subsystem
            .open_playback(None, &desired_spec, |spec| SquareWave {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 0.0,
                volume,
            })
            .map_err(|error| format!("SDL audio playback device failed: {error}"))?;
        Ok(Sound { device })
    }
}

pub fn play_sound(sound: &mut Sound) {
    match sound.device.status() {
        AudioStatus::Paused => sound.device.resume(),
        AudioStatus::Playing => {}
        AudioStatus::Stopped => sound.device.resume(),
    }
}
