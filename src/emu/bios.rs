//! Module for handling database of bios versions

pub const BIOS_SIZE : usize = 512 * 1024;
pub const BIOS_START: u32   = 0x1fc0_0000;
pub const BIOS_END  : u32   = BIOS_START + BIOS_SIZE as u32;

pub struct Bios {
    data: Vec<u8>
}

impl Bios {
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Bios { data }
    }

    pub fn get_rom(&self) -> &[u8] {
        &self.data
    }
}