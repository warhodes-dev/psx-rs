#![feature(array_chunks, let_chains)]
#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use] extern crate byte_unit;

use std::{path::Path, fs::File, io::Read};

use tracing_subscriber::{reload, filter, Registry, prelude::*};
pub static mut TRACING_RELOAD_HANDLE: Option<reload::Handle<filter::LevelFilter, Registry>> = None;

pub fn set_log_level(filter_level: filter::LevelFilter) {
    unsafe { 
        let _ = TRACING_RELOAD_HANDLE.as_ref().unwrap().modify(|filter| *filter = filter_level);
    }
}

use anyhow::{Result, anyhow};

pub mod config;
pub mod emu;
//pub mod sdl;

/// The interactive context of the PSX-RS emulator. Contains the emulation core and the frontend context
pub struct Context {
    pub psx: Box<emu::Psx>,
    //pub sdl: Option<sdl::SdlFrontend>,
}

impl Context {
    pub fn new(bios_path: &Path) -> Result<Context> {
        let bios = read_bios_file(bios_path)?;
        let psx = emu::Psx::new_from_bios(&bios);
        
        //let sdl = sdl::SdlFrontend::new()?;

        Ok( Context {
            psx: Box::new(psx),
            //sdl: None, //Some(sdl)
        })

        // add another comment
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            tracing::trace!("=== Instruction {:2} issued ===", self.psx.instructions_retired + 1);
            self.psx.step();
        }
    }
}

fn read_bios_file(path: &Path) -> Result<[u8; emu::bios::BIOS_SIZE]>{
    let mut file = File::open(path)?;
    let mut buf = [0u8; emu::bios::BIOS_SIZE as usize];
    file.read_exact(&mut buf)?;

    Ok(buf)
}
