use std::{
    path::Path, 
    fs::File, 
    io::Read
};
use anyhow::{
    Result, 
    anyhow
};

pub const BIOS_SIZE: usize = 512 * 1024;
pub const BIOS_START:  u32 = 0x1fc0_0000;

pub struct Bios {
    data: Vec<u8>
}

impl Bios {
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Bios { data }
    }

    /// Create a new Bios object from file at `path`
    pub fn _OLD_new(path: &Path) -> Result<Self> {
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

    pub fn get_rom(&self) -> &[u8] {
        &self.data
    }
}