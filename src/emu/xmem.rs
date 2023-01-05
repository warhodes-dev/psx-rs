use arrayvec::ArrayVec;

use super::bios::BIOS_SIZE;
use super::bios::BIOS_START;
pub const RAM_SIZE: usize = 2 * (1 << 20);
pub const RAM_START:  u32 = 0x0000_0000;

pub struct XMemory {
    mem: Vec<u32>,
}

impl XMemory {
    pub fn new() -> Self {
        let mem = vec![0xffff_ffff; RAM_SIZE]; 
        XMemory {
            mem,
        }
    }

    pub fn set_bios(&mut self, bios: &[u8; BIOS_SIZE]) {
        let mut bios_u32: Vec<u32> = Vec::new();
        for chunk in bios.chunks(4) {
            let quad = chunk.try_into().expect("Bios size must be a multiple of 4");
            let word = u32::from_le_bytes(quad);
            bios_u32.push(word);
        }
    }

    fn store(&mut self, offset: u32, val: u32) {
        let offset = offset as usize;
        self.mem[offset / 4] = val;
    }

    fn load(&self, offset: u32) -> u32 {
        let offset = offset as usize;
        self.mem[offset / 4]
    }

    pub fn ram_load(&self, offset: u32) ->  u32 {
        let offset = RAM_START + offset;
        self.load(offset)
    }

    pub fn ram_store(&mut self, offset: u32, val: u32) {
        let offset = RAM_START + offset;
        self.store(offset, val);
    }

    pub fn bios_load(&self, offset: u32) -> u32 {
        let offset = BIOS_START + offset;
        self.load(offset)
    }
}