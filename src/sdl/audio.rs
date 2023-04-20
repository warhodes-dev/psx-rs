use anyhow::{anyhow, Result};
use sdl2::{audio::{AudioDevice, AudioCallback, AudioSpecDesired}, Sdl};

struct BasicWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for BasicWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            if self.phase <= 0.5 {
                *x = self.volume;
            } else {
                *x = -self.volume;
            }
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

pub struct AudioDriver {
    device: AudioDevice<BasicWave>
}

impl AudioDriver {
    pub fn new(sdl_context: &Sdl) -> Result<Self> {
        let audio_subsystem = sdl_context.audio().map_err(|e| anyhow!(e))?;

        let desired_spec = AudioSpecDesired {
            freq: Some(44_100),
            channels: Some(1),
            samples: None,
        };

        let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
            log::debug!("Audio driver spec obtained: {:?}", spec);

            BasicWave {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.03,
            }
        }).map_err(|e| anyhow!(e))?;

        log::info!("SDL audio subsystem initialized");
        Ok( AudioDriver {
            device,
        })
    }
}