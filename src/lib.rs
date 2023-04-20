#![feature(array_chunks)]

use std::{path::Path, fs::File, io::Read};
use anyhow::Result;
use driver::{
    video::VideoDriver,
    audio::AudioDriver,
};

pub mod emu;
pub mod driver;

/// The interactive context of the PSX-RS emulator. Provides layer between emulation core and SDL context.
pub struct Context {
    pub psx: Box<emu::Psx>,
    pub video_driver: VideoDriver,
}

impl Context {
    pub fn new(bios_path: &Path) -> Result<Context, Box<dyn std::error::Error>> {
        let bios = load_bios_buffer(bios_path)?;
        let psx = emu::Psx::new_from_bios(&bios);

        let sdl_context = sdl2::init()?;
        let video_driver = VideoDriver::new(&sdl_context)?;
        let audio_driver = AudioDriver::new(&sdl_context)?;
        let input_driver = InputDriver::new(&sdl_context)?;

        Ok( Context {
            psx: Box::new(psx),
            video_driver,
        })
    }
}

fn load_bios_buffer(path: &Path) -> Result<[u8; emu::bios::BIOS_SIZE]>{
    let mut file = File::open(path)?;
    let mut buf = [0u8; emu::bios::BIOS_SIZE as usize];
    file.read_exact(&mut buf)?;

    Ok(buf)
}