use arrayvec::ArrayVec;

use super::bios::BIOS_SIZE;

const BIOS_OFFSET: usize = 0;
const RAM_OFFSET:  usize = BIOS_SIZE;
const RAM_SIZE:    usize = 1 << 21; // 2 MB 

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

    fn store(&mut self, offset: usize, val: u32) {
        self.mem[offset / 4] = val;
    }

    fn load(&self, offset: usize) -> u32 {
        self.mem[offset / 4]
    }

    pub fn ram_load(&self, offset: u32) ->  u32 {
        let offset = RAM_OFFSET + offset as usize;
        self.load(offset)
    }

    pub fn ram_store(&mut self, offset: u32, val: u32) {
        let offset = RAM_OFFSET + offset as usize;
        self.store(offset, val);
    }

    pub fn bios_load(&self, offset: u32) -> u32 {
        log::debug!("bios_load(offset) offset: {offset}");
        let offset = BIOS_OFFSET + offset as usize;
        self.load(offset)
    }
}