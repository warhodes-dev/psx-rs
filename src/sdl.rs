mod video;
mod audio;
mod input;

use anyhow::{anyhow, Result};
use crate::sdl::{
    video::VideoDriver,
    audio::AudioDriver,
    input::InputDriver,
};

pub struct SdlFrontend {
    _video: VideoDriver,
    _audio: AudioDriver,
    _input: InputDriver,
}

impl SdlFrontend {
    pub fn new() -> Result<SdlFrontend> {
        let sdl_context = sdl2::init().map_err(|e| anyhow!(e))?;
        let video = VideoDriver::new(&sdl_context)?;
        let audio = AudioDriver::new(&sdl_context)?;
        let input = InputDriver::new(&sdl_context)?;
        Ok(SdlFrontend { 
            _video: video, 
            _audio: audio,
            _input: input,
        })
    }
}