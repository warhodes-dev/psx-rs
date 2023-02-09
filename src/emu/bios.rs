//! Module for handling database of bios versions

pub const BIOS_SIZE : usize = 512 * 1024;
pub const BIOS_START: u32   = 0x1fc0_0000;
pub const BIOS_END  : u32   = BIOS_START + BIOS_SIZE as u32;

pub struct Bios {
    data: Vec<u32>
}

impl Bios {
    pub fn new(data: Vec<u32>) -> Self {
        Bios { data }
    }

    pub fn get_rom(&self) -> &[u32] {
        &self.data
    }

    fn load(&self, offset: usize) -> u32 {
        self.data[offset / 4]
    }

    pub fn load32(&self, offset: u32) -> u32 {
        log::trace!("bios_load(0x{offset:08x})");
        let offset = BIOS_OFFSET + offset as usize;
        self.load(offset)
    }
}