use std::{
    path::Path, 
    fs::File, 
    io::Read
};
use anyhow::{
    Result, 
    anyhow
};
use arrayvec::ArrayVec;

pub const BIOS_SIZE:  u32 = 512 * 1024;
pub const BIOS_START: u32 = 0xbfc0_0000;

pub struct Bios {
    data: ArrayVec<u8, {BIOS_SIZE as usize} >
}

impl Bios {
    /// Create a new Bios object from file at `path`
    pub fn new(path: &Path) -> Result<Self> {
        let file = File::open(path)?;

        let mut data_buf = Vec::<u8>::new();

        let bytes_read = file.take(BIOS_SIZE as u64).read_to_end(&mut data_buf)?;

        if bytes_read != BIOS_SIZE as usize {
            return Err(anyhow!("Invalid BIOS size (read {} bytes)", bytes_read));
        }

        let data = data_buf.into_iter().collect();

        Ok( Bios {
            data,
        })
    }

    // ASK: Is this the best place for this kind of implementation?
    /// Load a 4 byte little endian word from `offset`
    pub fn load32(&self, offset: u32) -> u32 {
        let offset = offset as usize;

        let b0 = self.data[offset + 0] as u32;
        let b1 = self.data[offset + 1] as u32;
        let b2 = self.data[offset + 2] as u32;
        let b3 = self.data[offset + 3] as u32;

        b0 | (b1 << 8) | (b2 << 16) | (b3 << 24)
    }
}