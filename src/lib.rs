use std::{path::Path, fs::File, io::Read};
use anyhow::{Result};

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

    fn load_bios(path: &Path) -> Result<emu::bios::Bios> {
        let mut file = File::open(path)?;
        //let mut bios_buf = ArrayVec::from(vec![0u8; emu::bios::BIOS_SIZE as usize].into_boxed_slice());
        let mut bios_buf = vec![0u8; emu::bios::BIOS_SIZE as usize];
        file.read_exact(&mut bios_buf)?;

        Ok(emu::bios::Bios::from_bytes(bios_buf))
    }
}

