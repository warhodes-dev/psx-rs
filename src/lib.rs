#![feature(array_chunks)]

use std::{path::Path, fs::File, io::Read};
use anyhow::Result;
use driver::{
    video::VideoDriver,
};

pub mod emu;
pub mod driver;

pub struct Context {
    pub psx: Box<emu::Psx>,
    pub video_driver: Option<VideoDriver>,
}

pub struct ContextBuilder {
    pub psx: emu::Psx,
    pub video_driver: Option<VideoDriver>,
}

impl ContextBuilder {
    pub fn new(bios_path: &Path) -> Result<ContextBuilder> {
        let bios = load_bios_buffer(bios_path)?;
        let psx = emu::Psx::new_from_bios(&bios);
        Ok( ContextBuilder {
            psx,
            video_driver: None,
        })
    }

    pub fn with_sdl2(mut self) -> Result<ContextBuilder, Box<dyn std::error::Error>> {
        let sdl_context = sdl2::init()?;
        let video_driver = VideoDriver::new(&sdl_context)?;
        self.video_driver = Some(video_driver);
        Ok(self)
    }

    pub fn init(self) -> Result<Context> {
        Ok( Context {
            psx: Box::new(self.psx),
            video_driver: self.video_driver,
        })
    }
}

fn load_bios_buffer(path: &Path) -> Result<[u8; emu::bios::BIOS_SIZE]>{
    let mut file = File::open(path)?;
    let mut buf = [0u8; emu::bios::BIOS_SIZE as usize];
    file.read_exact(&mut buf)?;

    Ok(buf)
}