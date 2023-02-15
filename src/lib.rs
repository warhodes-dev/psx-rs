use std::{path::Path, fs::File, io::Read};
use anyhow::Result;

pub mod emu;

pub struct Context {
    pub psx: Box<emu::Psx>,
}

impl Context {
    pub fn new_no_disc(bios_path: &Path) -> Result<Context> {
        let bios = Context::load_bios_buffer(bios_path)?;
        let psx = emu::Psx::new_from_bios(&bios);
        Ok( Context {
            psx: Box::new(psx),
        })
    }

    fn load_bios_buffer(path: &Path) -> Result<[u8; emu::bios::BIOS_SIZE]>{
        let mut file = File::open(path)?;
        let mut buf = [0u8; emu::bios::BIOS_SIZE as usize];
        file.read_exact(&mut buf)?;

        Ok(buf)
    }
}
