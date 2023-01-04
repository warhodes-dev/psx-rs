use std::{path::Path, fs::File, io::Read, thread::sleep};
use anyhow::{Result, anyhow};
use arrayvec::ArrayVec;

pub mod emu;

pub struct Context {
    pub psx: Box<emu::Psx>,
}

impl Context {
    pub fn new_no_disc(bios_path: &Path) -> Result<Context> {
        let bios = Context::load_bios(bios_path)?;
        let psx = emu::Psx::new_from_bios(bios);
        Ok( Context {
            psx: Box::new(psx),
        })
    }

    pub fn _new_from_disc(disc_path: &Path, bios_path: &Path) -> Result<Box<emu::Psx>> {
        let disc = Context::_load_disc(disc_path);
        let bios = Context::load_bios(bios_path)?;
        let psx = emu::Psx::_new_from_disc(disc, bios);
        Ok(Box::new(psx))
    }

    fn load_bios(path: &Path) -> Result<emu::bios::Bios> {
        let mut file = File::open(path)?;
        //let mut bios_buf = ArrayVec::from(vec![0u8; emu::bios::BIOS_SIZE as usize].into_boxed_slice());
        let mut bios_buf = vec![0u8; emu::bios::BIOS_SIZE as usize];
        file.read_exact(&mut bios_buf)?;

        Ok(emu::bios::Bios::from_bytes(bios_buf))
    }

    fn _load_disc(path: &Path) {
        todo!();
    }
}

