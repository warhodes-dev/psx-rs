#![feature(array_chunks, let_chains)]
#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use] extern crate byte_unit;

use std::{path::Path, fs::File, io::Read};
use anyhow::Result;

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
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            log::trace!("=== Instruction {:2} issued ===", self.psx.instruction_cnt);
            self.psx.instruction_cnt += 1;
            crate::emu::cpu::handle_next_instruction(&mut self.psx)?;
        }
    }
}

fn read_bios_file(path: &Path) -> Result<[u8; emu::bios::BIOS_SIZE]>{
    let mut file = File::open(path)?;
    let mut buf = [0u8; emu::bios::BIOS_SIZE as usize];
    file.read_exact(&mut buf)?;

    Ok(buf)
}